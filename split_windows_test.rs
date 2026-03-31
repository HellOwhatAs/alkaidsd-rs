pub fn split_windows_4(succ: impl Fn(usize) -> usize + Copy, slices: [(usize, usize); 4]) {
    let w33 = |mut wit: SlicesSplitWindowsIter<_, 4, 3, 3>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left: [_; 4] = concat_arrays!(
                (slice_array!(slices, 0..3), 3),
                ([(slices[3].0, split1.0)], 1)
            );
            let mid = [(split1.1, split2.0)];
            let right = [(split2.1, slices[3].1)];
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                unreachable!();
            }
            (true, false) => {
                unreachable!();
            }
            (false, true) => {
                let left = concat_arrays!(
                    (slice_array!(slices, 0..3), 3),
                    ([(slices[3].0, wit.current1)], 1)
                );
                let mid = [(succ(wit.current1), slices[3].1)];
                let right: [(usize, usize); _] = [];
                println!("{:?}", (left, mid, right));
            }
            (false, false) => unreachable!(),
        }
    };

    let w23 = |mut wit: SlicesSplitWindowsIter<_, 4, 2, 3>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left: [_; 3] = concat_arrays!(
                (slice_array!(slices, 0..2), 2),
                ([(slices[2].0, split1.0)], 1)
            );
            let mid = [(split1.1, slices[2].1), (slices[3].0, split2.0)];
            let right = [(split2.1, slices[3].1)];
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                let left = slice_array!(slices, 0..3);
                let mid = slice_array!(slices, 3..4);
                let right: [(usize, usize); _] = [];
                println!("{:?}", (left, mid, right));
            }
            (true, false) => {
                let left = slice_array!(slices, 0..3);
                let mid = [(slices[3].0, wit.current1)];
                let right = [(succ(wit.current1), slices[3].1)];
                println!("{:?}", (left, mid, right));
                let wit = SlicesSplitWindowsIter::<_, _, 3, 3>::from10(wit);
                w33(wit);
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
            }
            (false, false) => unreachable!(),
        }
    };

    let w13 = |mut wit: SlicesSplitWindowsIter<_, 4, 1, 3>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left: [_; 2] = concat_arrays!(
                (slice_array!(slices, 0..1), 1),
                ([(slices[1].0, split1.0)], 1)
            );
            let mid: [_; 3] = concat_arrays!(
                ([(split1.1, slices[1].1)], 1),
                (slice_array!(slices, 2..3), 1),
                ([(slices[3].0, split2.0)], 1)
            );
            let right = [(split2.1, slices[3].1)];
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 2, 3>::from10(wit);
                w23(wit);
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
            }
            (false, false) => unreachable!(),
        }
    };

    let w22 = |mut wit: SlicesSplitWindowsIter<_, 4, 2, 2>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left: [_; 3] = concat_arrays!(
                (slice_array!(slices, 0..2), 2),
                ([(slices[2].0, split1.0)], 1)
            );
            let mid = [(split1.1, split2.0)];
            let right: [_; 2] = concat_arrays!(
                ([(split2.1, slices[2].1)], 1),
                (slice_array!(slices, 3..4), 1)
            );
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 3, 3>::from11(wit);
                w33(wit);
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 2, 3>::from01(wit);
                w23(wit);
            }
            (false, false) => unreachable!(),
        }
    };

    let w03 = |mut wit: SlicesSplitWindowsIter<_, 4, 0, 3>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left = [(slices[0].0, split1.0)];
            let mid: [_; 4] = concat_arrays!(
                ([(split1.1, slices[0].1)], 1),
                (slice_array!(slices, 1..3), 2),
                ([(slices[3].0, split2.0)], 1)
            );
            let right = [(split2.1, slices[3].1)];
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 3>::from10(wit);
                w13(wit);
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
            }
            (false, false) => unreachable!(),
        }
    };

    let w12 = |mut wit: SlicesSplitWindowsIter<_, 4, 1, 2>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left: [_; 2] = concat_arrays!(
                (slice_array!(slices, 0..1), 1),
                ([(slices[1].0, split1.0)], 1)
            );
            let mid = [(split1.1, slices[1].1), (slices[2].0, split2.0)];
            let right: [_; 2] = concat_arrays!(
                ([(split2.1, slices[2].1)], 1),
                (slice_array!(slices, 3..4), 1)
            );
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 2, 3>::from11(wit);
                w23(wit);
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 2, 2>::from10(wit);
                w22(wit);
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 3>::from01(wit);
                w13(wit);
            }
            (false, false) => unreachable!(),
        }
    };

    let w02 = |mut wit: SlicesSplitWindowsIter<_, 4, 0, 2>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left = [(slices[0].0, split1.0)];
            let mid: [_; 3] = concat_arrays!(
                ([(split1.1, slices[0].1)], 1),
                (slice_array!(slices, 1..2), 1),
                ([(slices[2].0, split2.0)], 1)
            );
            let right: [_; 2] = concat_arrays!(
                ([(split2.1, slices[2].1)], 1),
                (slice_array!(slices, 3..4), 1)
            );
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 3>::from11(wit);
                w13(wit);
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from10(wit);
                w12(wit);
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 0, 3>::from01(wit);
                w03(wit);
            }
            (false, false) => unreachable!(),
        }
    };

    let w11 = |mut wit: SlicesSplitWindowsIter<_, 4, 1, 1>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left: [_; 2] = concat_arrays!(
                (slice_array!(slices, 0..1), 1),
                ([(slices[1].0, split1.0)], 1)
            );
            let mid = [(split1.1, split2.0)];
            let right: [_; 3] = concat_arrays!(
                ([(split2.1, slices[1].1)], 1),
                (slice_array!(slices, 2..4), 2)
            );
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 2, 2>::from11(wit);
                w22(wit);
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from01(wit);
                w12(wit);
            }
            (false, false) => unreachable!(),
        }
    };

    let w01 = |mut wit: SlicesSplitWindowsIter<_, 4, 0, 1>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left = [(slices[0].0, split1.0)];
            let mid = [(split1.1, slices[0].1), (slices[1].0, split2.0)];
            let right: [_; 3] = concat_arrays!(
                ([(split2.1, slices[1].1)], 1),
                (slice_array!(slices, 2..4), 2)
            );
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from11(wit);
                w12(wit);
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 1>::from10(wit);
                w11(wit);
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 0, 2>::from01(wit);
                w02(wit);
            }
            (false, false) => unreachable!(),
        }
    };

    let w00 = |mut wit: SlicesSplitWindowsIter<_, 4, 0, 0>| {
        let slices = wit.slices;
        for (split1, split2) in &mut wit {
            let left = [(slices[0].0, split1.0)];
            let mid = [(split1.1, split2.0)];
            let right: [_; 4] = concat_arrays!(
                ([(split2.1, slices[0].1)], 1),
                (slice_array!(slices, 1..4), 3)
            );
            println!("{:?}", (left, mid, right));
        }
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 1>::from11(wit);
                w11(wit);
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 0, 1>::from01(wit);
                w01(wit);
            }
            (false, false) => unreachable!(),
        }
    };

    // --- entry point ---
    match SlicesSplitWindowsIter::<_, _, 0, 0>::new(succ, slices, 3) {
        Ok(wit) => w00(wit),
        Err(cnt) => match SlicesSplitWindowsIter::<_, _, 0, 1>::new(succ, slices, cnt) {
            Ok(wit) => w01(wit),
            Err(cnt) => match SlicesSplitWindowsIter::<_, _, 0, 2>::new(succ, slices, cnt) {
                Ok(wit) => w02(wit),
                Err(cnt) => match SlicesSplitWindowsIter::<_, _, 0, 3>::new(succ, slices, cnt) {
                    Ok(wit) => w03(wit),
                    Err(_) => {} // window larger than all slices combined
                },
            },
        },
    }
}
