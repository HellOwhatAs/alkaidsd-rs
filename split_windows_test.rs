pub trait Slices3Handler {
    fn call<const N1: usize, const N2: usize, const N3: usize>(
        &mut self,
        slice3: ([(usize, usize); N1], [(usize, usize); N2], [(usize, usize); N3]),
    );
}

pub fn split_windows_4<H: Slices3Handler>(h: &mut H, succ: impl Fn(usize) -> usize + Copy, slices: [(usize, usize); 4]) {
let w33 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 3, 3>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left: [_; 4] = concat_arrays!((slice_array!(slices, 0..3), 3), ([(slices[3].0, split1.0)], 1));
        let mid = [(split1.1, split2.0)];
        let right = [(split2.1, slices[3].1)];
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left: [_; 4] = concat_arrays!((slice_array!(slices, 0..3), 3), ([(slices[3].0, split1.0)], 1));
            let mid = [(split1.1, slices[3].1)];
            let right: [(usize, usize); 0] = [];
            h.call((left, mid, right));
        }
        _ => unreachable!(),
    }
};

let w23 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 2, 3>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left: [_; 3] = concat_arrays!((slice_array!(slices, 0..2), 2), ([(slices[2].0, split1.0)], 1));
        let mid = [(split1.1, slices[2].1), (slices[3].0, split2.0)];
        let right = [(split2.1, slices[3].1)];
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (true, true) => {
            let left = slice_array!(slices, 0..3);
            let mid = [slices[3]];
            let right: [(usize, usize); 0] = [];
            h.call((left, mid, right));
        }
        (true, false) => {
            let split2 = (wit.current2, succ(wit.current2));
            let left = slice_array!(slices, 0..3);
            let mid = [(slices[3].0, split2.0)];
            let right = [(split2.1, slices[3].1)];
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 3, 3>::from10(wit);
            w33(h, wit);
        }
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left: [_; 3] = concat_arrays!((slice_array!(slices, 0..2), 2), ([(slices[2].0, split1.0)], 1));
            let mid = [(split1.1, slices[2].1), slices[3]];
            let right: [(usize, usize); 0] = [];
            h.call((left, mid, right));
        }
        _ => unreachable!(),
    }
};

let w13 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 1, 3>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left: [_; 2] = concat_arrays!((slice_array!(slices, 0..1), 1), ([(slices[1].0, split1.0)], 1));
        let mid: [_; 3] = concat_arrays!(([(split1.1, slices[1].1)], 1), (slice_array!(slices, 2..3), 1), ([(slices[3].0, split2.0)], 1));
        let right = [(split2.1, slices[3].1)];
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (true, true) => {
            let left = slice_array!(slices, 0..2);
            let mid = slice_array!(slices, 2..4);
            let right: [(usize, usize); 0] = [];
            h.call((left, mid, right));
        }
        (true, false) => {
            let split2 = (wit.current2, succ(wit.current2));
            let left = slice_array!(slices, 0..2);
            let mid = concat_arrays!((slice_array!(slices, 2..3), 1), ([(slices[3].0, split2.0)], 1));
            let right = [(split2.1, slices[3].1)];
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 2, 3>::from10(wit);
            w23(h, wit);
        }
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left: [_; 2] = concat_arrays!((slice_array!(slices, 0..1), 1), ([(slices[1].0, split1.0)], 1));
            let mid = concat_arrays!(([(split1.1, slices[1].1)], 1), (slice_array!(slices, 2..4), 2));
            let right: [(usize, usize); 0] = [];
            h.call((left, mid, right));
        }
        _ => unreachable!(),
    }
};

let w22 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 2, 2>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left: [_; 3] = concat_arrays!((slice_array!(slices, 0..2), 2), ([(slices[2].0, split1.0)], 1));
        let mid = [(split1.1, split2.0)];
        let right: [_; 2] = concat_arrays!(([(split2.1, slices[2].1)], 1), (slice_array!(slices, 3..4), 1));
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left: [_; 3] = concat_arrays!((slice_array!(slices, 0..2), 2), ([(slices[2].0, split1.0)], 1));
            let mid = [(split1.1, slices[2].1)];
            let right = slice_array!(slices, 3..4);
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 2, 3>::from01(wit);
            w23(h, wit);
        }
        _ => unreachable!(),
    }
};

let w03 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 0, 3>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left = [(slices[0].0, split1.0)];
        let mid: [_; 4] = concat_arrays!(([(split1.1, slices[0].1)], 1), (slice_array!(slices, 1..3), 2), ([(slices[3].0, split2.0)], 1));
        let right = [(split2.1, slices[3].1)];
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (true, true) => {
            let left = [slices[0]];
            let mid = slice_array!(slices, 1..4);
            let right: [(usize, usize); 0] = [];
            h.call((left, mid, right));
        }
        (true, false) => {
            let split2 = (wit.current2, succ(wit.current2));
            let left = [slices[0]];
            let mid = concat_arrays!((slice_array!(slices, 1..3), 2), ([(slices[3].0, split2.0)], 1));
            let right = [(split2.1, slices[3].1)];
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 1, 3>::from10(wit);
            w13(h, wit);
        }
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left = [(slices[0].0, split1.0)];
            let mid = concat_arrays!(([(split1.1, slices[0].1)], 1), (slice_array!(slices, 1..4), 3));
            let right: [(usize, usize); 0] = [];
            h.call((left, mid, right));
        }
        _ => unreachable!(),
    }
};

let w12 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 1, 2>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left: [_; 2] = concat_arrays!((slice_array!(slices, 0..1), 1), ([(slices[1].0, split1.0)], 1));
        let mid = [(split1.1, slices[1].1), (slices[2].0, split2.0)];
        let right: [_; 2] = concat_arrays!(([(split2.1, slices[2].1)], 1), (slice_array!(slices, 3..4), 1));
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (true, true) => {
            let left = slice_array!(slices, 0..2);
            let mid = [slices[2]];
            let right = slice_array!(slices, 3..4);
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 2, 3>::from11(wit);
            w23(h, wit);
        }
        (true, false) => {
            let split2 = (wit.current2, succ(wit.current2));
            let left = slice_array!(slices, 0..2);
            let mid = [(slices[2].0, split2.0)];
            let right: [_; 2] = concat_arrays!(([(split2.1, slices[2].1)], 1), (slice_array!(slices, 3..4), 1));
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 2, 2>::from10(wit);
            w22(h, wit);
        }
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left: [_; 2] = concat_arrays!((slice_array!(slices, 0..1), 1), ([(slices[1].0, split1.0)], 1));
            let mid = [(split1.1, slices[1].1), slices[2]];
            let right = slice_array!(slices, 3..4);
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 1, 3>::from01(wit);
            w13(h, wit);
        }
        _ => unreachable!(),
    }
};

let w02 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 0, 2>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left = [(slices[0].0, split1.0)];
        let mid: [_; 3] = concat_arrays!(([(split1.1, slices[0].1)], 1), (slice_array!(slices, 1..2), 1), ([(slices[2].0, split2.0)], 1));
        let right: [_; 2] = concat_arrays!(([(split2.1, slices[2].1)], 1), (slice_array!(slices, 3..4), 1));
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (true, true) => {
            let left = [slices[0]];
            let mid = slice_array!(slices, 1..3);
            let right = slice_array!(slices, 3..4);
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 1, 3>::from11(wit);
            w13(h, wit);
        }
        (true, false) => {
            let split2 = (wit.current2, succ(wit.current2));
            let left = [slices[0]];
            let mid = concat_arrays!((slice_array!(slices, 1..2), 1), ([(slices[2].0, split2.0)], 1));
            let right: [_; 2] = concat_arrays!(([(split2.1, slices[2].1)], 1), (slice_array!(slices, 3..4), 1));
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from10(wit);
            w12(h, wit);
        }
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left = [(slices[0].0, split1.0)];
            let mid = concat_arrays!(([(split1.1, slices[0].1)], 1), (slice_array!(slices, 1..3), 2));
            let right = slice_array!(slices, 3..4);
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 0, 3>::from01(wit);
            w03(h, wit);
        }
        _ => unreachable!(),
    }
};

let w11 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 1, 1>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left: [_; 2] = concat_arrays!((slice_array!(slices, 0..1), 1), ([(slices[1].0, split1.0)], 1));
        let mid = [(split1.1, split2.0)];
        let right: [_; 3] = concat_arrays!(([(split2.1, slices[1].1)], 1), (slice_array!(slices, 2..4), 2));
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left: [_; 2] = concat_arrays!((slice_array!(slices, 0..1), 1), ([(slices[1].0, split1.0)], 1));
            let mid = [(split1.1, slices[1].1)];
            let right = slice_array!(slices, 2..4);
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from01(wit);
            w12(h, wit);
        }
        _ => unreachable!(),
    }
};

let w01 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 0, 1>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left = [(slices[0].0, split1.0)];
        let mid = [(split1.1, slices[0].1), (slices[1].0, split2.0)];
        let right: [_; 3] = concat_arrays!(([(split2.1, slices[1].1)], 1), (slice_array!(slices, 2..4), 2));
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (true, true) => {
            let left = [slices[0]];
            let mid = [slices[1]];
            let right = slice_array!(slices, 2..4);
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from11(wit);
            w12(h, wit);
        }
        (true, false) => {
            let split2 = (wit.current2, succ(wit.current2));
            let left = [slices[0]];
            let mid = [(slices[1].0, split2.0)];
            let right: [_; 3] = concat_arrays!(([(split2.1, slices[1].1)], 1), (slice_array!(slices, 2..4), 2));
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 1, 1>::from10(wit);
            w11(h, wit);
        }
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left = [(slices[0].0, split1.0)];
            let mid = [(split1.1, slices[0].1), slices[1]];
            let right = slice_array!(slices, 2..4);
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 0, 2>::from01(wit);
            w02(h, wit);
        }
        _ => unreachable!(),
    }
};

let w00 = |h: &mut H, mut wit: SlicesSplitWindowsIter<_, 4, 0, 0>| {
    let slices = wit.slices;
    for (split1, split2) in &mut wit {
        let left = [(slices[0].0, split1.0)];
        let mid = [(split1.1, split2.0)];
        let right: [_; 4] = concat_arrays!(([(split2.1, slices[0].1)], 1), (slice_array!(slices, 1..4), 3));
        h.call((left, mid, right));
    }
    match (wit.first_exhausted(), wit.second_exhausted()) {
        (false, true) => {
            let split1 = (wit.current1, succ(wit.current1));
            let left = [(slices[0].0, split1.0)];
            let mid = [(split1.1, slices[0].1)];
            let right = slice_array!(slices, 1..4);
            h.call((left, mid, right));
            let wit = SlicesSplitWindowsIter::<_, _, 0, 1>::from01(wit);
            w01(h, wit);
        }
        _ => unreachable!(),
    }
};

// --- entry point ---
match SlicesSplitWindowsIterBuilder::<_, _, 0, 0>::new(succ, slices, 3) {
    Ok(wib) => {
        let left: [(usize, usize); _] = [];
        let mid = [(slices[0].0, wib.current1)];
        let right = concat_arrays!(([(wib.current2, slices[0].1)], 1), (slice_array!(slices, 1..4), 3));
        h.call((left, mid, right));
        let wit = wib.build();
        w00(h, wit);
    }
    Err(cnt) =>
    match SlicesSplitWindowsIterBuilder::<_, _, 0, 1>::new(succ, slices, cnt) {
        Ok(wib) => {
            if wib.current2 == slices[1].0 {
                let left: [(usize, usize); _] = [];
                let mid = slice_array!(slices, 0..1);
                let right = slice_array!(slices, 1..4);
                h.call((left, mid, right));
            } else {
                let left: [(usize, usize); _] = [];
                let split2 = (wib.current1, wib.current2);
                let mid: [_; 2] = concat_arrays!((slice_array!(slices, 0..1), 1), ([(slices[1].0, split2.0)], 1));
                let right: [_; 3] = concat_arrays!(([(split2.1, slices[1].1)], 1), (slice_array!(slices, 2..4), 2));
                h.call((left, mid, right));
            }
            let wit = wib.build();
            w01(h, wit);
        }
        Err(cnt) =>
        match SlicesSplitWindowsIterBuilder::<_, _, 0, 2>::new(succ, slices, cnt) {
            Ok(wib) => {
                if wib.current2 == slices[2].0 {
                    let left: [(usize, usize); _] = [];
                    let mid = slice_array!(slices, 0..2);
                    let right = slice_array!(slices, 2..4);
                    h.call((left, mid, right));
                } else {
                    let left: [(usize, usize); _] = [];
                    let split2 = (wib.current1, wib.current2);
                    let mid: [_; 3] = concat_arrays!((slice_array!(slices, 0..2), 2), ([(slices[2].0, split2.0)], 1));
                    let right: [_; 2] = concat_arrays!(([(split2.1, slices[2].1)], 1), (slice_array!(slices, 3..4), 1));
                    h.call((left, mid, right));
                }
                let wit = wib.build();
                w02(h, wit);
            }
            Err(cnt) =>
            match SlicesSplitWindowsIterBuilder::<_, _, 0, 3>::new(succ, slices, cnt) {
                Ok(wib) => {
                    if wib.current2 == slices[3].0 {
                        let left: [(usize, usize); _] = [];
                        let mid = slice_array!(slices, 0..3);
                        let right = slice_array!(slices, 3..4);
                        h.call((left, mid, right));
                    } else {
                        let left: [(usize, usize); _] = [];
                        let split2 = (wib.current1, wib.current2);
                        let mid: [_; 4] = concat_arrays!((slice_array!(slices, 0..3), 3), ([(slices[3].0, split2.0)], 1));
                        let right = [(split2.1, slices[3].1)];
                        h.call((left, mid, right));
                    }
                    let wit = wib.build();
                    w03(h, wit);
                }
                Err(_) => {} // window larger than all slices combined
            }
        }
    }
}
}