#!/usr/bin/env python3
"""
Generates the closure-based state machine for SlicesSplitWindowsIter inside a test function.

Each closure w{a}{b} handles the state where the first pointer is in slice `a`
and the second pointer is in slice `b`. Closures are emitted in reverse
topological order (highest a+b first) so that each closure's captured
dependencies are already defined when the capture happens.
"""


def make_left(a: int, exhausted: bool = False) -> str:
    if a == 0:
        if exhausted:
            return "let left = [slices[0]];"
        return "let left = [(slices[0].0, split1.0)];"
    if exhausted:
        return f"let left = slice_array!(slices, 0..{a + 1});"
    return (
        f"let left: [_; {a + 1}] = concat_arrays!("
        f"(slice_array!(slices, 0..{a}), {a}), "
        f"([(slices[{a}].0, split1.0)], 1));"
    )


def make_mid(
    a: int,
    b: int,
    first_exhausted: bool = False,
    second_exhausted: bool = False,
) -> str:
    if a == b:
        # assert not first_exhausted
        if second_exhausted:
            return f"let mid = [(split1.1, slices[{b}].1)];"
        return "let mid = [(split1.1, split2.0)];"
    if b == a + 1:
        match (first_exhausted, second_exhausted):
            case (True, True):
                return f"let mid = [slices[{b}]];"
            case (True, False):
                return f"let mid = [(slices[{b}].0, split2.0)];"
            case (False, True):
                return f"let mid = [(split1.1, slices[{a}].1), slices[{b}]];"
            case (False, False):
                return (
                    f"let mid = [(split1.1, slices[{a}].1), (slices[{b}].0, split2.0)];"
                )
    match (first_exhausted, second_exhausted):
        case (True, True):
            return f"let mid = slice_array!(slices, {a + 1}..{b + 1});"
        case (True, False):
            return f"let mid = concat_arrays!((slice_array!(slices, {a + 1}..{b}), {b - a - 1}), ([(slices[{b}].0, split2.0)], 1));"
        case (False, True):
            return f"let mid = concat_arrays!(([(split1.1, slices[{a}].1)], 1), (slice_array!(slices, {a + 1}..{b + 1}), {b - a}));"
        case (False, False):
            return (
                f"let mid: [_; {b - a + 1}] = concat_arrays!("
                f"([(split1.1, slices[{a}].1)], 1), "
                f"(slice_array!(slices, {a + 1}..{b}), {b - a - 1}), "
                f"([(slices[{b}].0, split2.0)], 1));"
            )


def make_right(b: int, N: int, exhausted: bool = False) -> str:
    if b == N - 1:
        if exhausted:
            return "let right: [(usize, usize); 0] = [];"
        return f"let right = [(split2.1, slices[{b}].1)];"
    if exhausted:
        return f"let right = slice_array!(slices, {b + 1}..{N});"
    return (
        f"let right: [_; {N - b}] = concat_arrays!("
        f"([(split2.1, slices[{b}].1)], 1), "
        f"(slice_array!(slices, {b + 1}..{N}), {N - b - 1}));"
    )


def split_windows(N: int, window: int) -> str:
    lines = [
        "pub fn split_windows_4(succ: impl Fn(usize) -> usize + Copy, slices: [(usize, usize); 4]) {"
    ]

    # All (a, b) states, sorted highest a+b first (topological order for closures)
    all_states = sorted(
        [(a, b) for a in range(N) for b in range(N) if a <= b],
        key=lambda s: -(s[0] + s[1]),
    )

    for a, b in all_states:
        # Next state for each exhaustion pattern, None if out of bounds (terminal)
        tt = (a + 1, b + 1) if a + 1 < N and b + 1 < N else None  # both exhausted
        tf = (a + 1, b) if a + 1 < N else None  # first exhausted
        ft = (a, b + 1) if b + 1 < N else None  # second exhausted

        lines += [
            f"let w{a}{b} = |mut wit: SlicesSplitWindowsIter<_, {N}, {a}, {b}>| {{",
            "    let slices = wit.slices;",
            "    for (split1, split2) in &mut wit {",
            "        " + make_left(a),
            "        " + make_mid(a, b),
            "        " + make_right(b, N),
            '        println!("{:?}\\t{:?}\\t{:?}", left, mid, right);',
            "    }",
            "    match (wit.first_exhausted(), wit.second_exhausted()) {",
        ]

        for cond, next_state, from_fn in [
            ((True, True), tt, "from11"),
            ((True, False), tf, "from10"),
            ((False, True), ft, "from01"),
        ]:
            if a == b and cond[0]:
                continue  # can't exhaust the first pointer if both pointers are in the same slice
            lines.append(f"        ({', '.join(str(c).lower() for c in cond)}) => {{")
            if not cond[0]:
                lines.append(
                    "            let split1 = (wit.current1, succ(wit.current1));"
                )
            if not cond[1]:
                lines.append(
                    "            let split2 = (wit.current2, succ(wit.current2));"
                )
            lines.append("            " + make_left(a, exhausted=cond[0]))
            lines.append(
                "            "
                + make_mid(a, b, first_exhausted=cond[0], second_exhausted=cond[1])
            )
            lines.append("            " + make_right(b, N, exhausted=cond[1]))
            lines.append(
                '            println!("{:?}\\t{:?}\\t{:?}", left, mid, right);'
            )
            if next_state is not None:
                na, nb = next_state
                if na <= nb:
                    lines.append(
                        f"            let wit = SlicesSplitWindowsIter::<_, _, {na}, {nb}>::{from_fn}(wit);"
                    )
                    lines.append(f"            w{na}{nb}(wit);")
            lines.append("        }")

        lines += [
            "        _ => unreachable!(),",
            "    }",
            "};",
            "",
        ]

    # Initialization: try to place the second pointer `window` steps ahead,
    # starting from slice 0 and falling through to slice 1, 2, ... if the
    # current slice is too short.
    lines.append("// --- entry point ---")
    for j in range(N):
        indent = "    " * j
        cnt_arg = str(window) if j == 0 else "cnt"
        lines.append(
            f"{indent}match SlicesSplitWindowsIterBuilder::<_, _, 0, {j}>::new(succ, slices, {cnt_arg}) {{"
        )
        lines.append(f"{indent}    Ok(wib) => {{")
        if j == 0:
            lines.append(f"{indent}        let left: [(usize, usize); _] = [];")
            lines.append(f"{indent}        let mid = [(slices[0].0, wib.current1)];")
            lines.append(
                f"{indent}        let right = concat_arrays!(([(wib.current2, slices[0].1)], 1), (slice_array!(slices, 1..{N}), {N - 1}));"
            )
            lines.append(
                f'{indent}        println!("{{:?}}\\t{{:?}}\\t{{:?}}", left, mid, right);'
            )
        else:
            lines.append(f"{indent}        if wib.current2 == slices[{j}].0 {{")
            lines.append(f"{indent}            let left: [(usize, usize); _] = [];")
            lines.append(f"{indent}            let mid = slice_array!(slices, 0..{j});")
            lines.append(
                f"{indent}            let right = slice_array!(slices, {j}..{N});"
            )
            lines.append(
                f'{indent}            println!("{{:?}}\\t{{:?}}\\t{{:?}}", left, mid, right);'
            )
            lines.append(f"{indent}        }} else {{")
            lines.append(f"{indent}            let left: [(usize, usize); _] = [];")
            lines.append(
                f"{indent}            let split2 = (wib.current1, wib.current2);"
            )
            lines.append(
                f"{indent}            let mid: [_; {j + 1}] = concat_arrays!("
                f"(slice_array!(slices, 0..{j}), {j}), "
                f"([(slices[{j}].0, split2.0)], 1));"
            )
            lines.append(f"{indent}            " + make_right(j, N, False))
            lines.append(
                f'{indent}            println!("{{:?}}\\t{{:?}}\\t{{:?}}", left, mid, right);'
            )
            lines.append(f"{indent}        }}")
        lines.append(f"{indent}        let wit = wib.build();")
        lines.append(f"{indent}        w0{j}(wit);")
        lines.append(f"{indent}    }}")
        if j < N - 1:
            lines.append(f"{indent}    Err(cnt) =>")
        else:
            lines.append(
                f"{indent}    Err(_) => {{}} // window larger than all slices combined"
            )

    # Close the nested matches from innermost to outermost
    for j in range(N - 1, -1, -1):
        lines.append("    " * j + "}")

    lines.append("}")
    return "\n".join(lines)


if __name__ == "__main__":
    with open("./split_windows_test.rs", "w", encoding="utf-8") as f:
        f.write(split_windows(N=4, window=3))
