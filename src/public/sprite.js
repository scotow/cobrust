const FRAMES_MAPPING = (() => {
    const HEAD = 1;
    const NOT_HEAD = NOT_SET = 0;
    const IN = 6;
    const OUT = 1;
    const UP = 9;                                          // 0b0100_1
    const DOWN = 17;                                       // 0b1000_1
    const LEFT = 3;                                        // 0b0001_1
    const RIGHT = 5;                                       // 0b0010_1

    return new Map([
        [DOWN << IN | RIGHT << OUT | NOT_HEAD, 1],         // Going down + right corner.
        [LEFT << IN | UP << OUT | NOT_HEAD, 1],            // Going left + up corner.
        [DOWN << IN | LEFT << OUT | NOT_HEAD, 2],          // Going down + left corner.
        [RIGHT << IN | UP << OUT | NOT_HEAD, 2],           // Going right + up corner.
        [UP << IN | RIGHT << OUT | NOT_HEAD, 3],           // Going up + right corner.
        [LEFT << IN | DOWN << OUT | NOT_HEAD, 3],          // Going left + down corner.
        [UP << IN | LEFT << OUT | NOT_HEAD, 4],            // Going up + left corner.
        [RIGHT << IN | DOWN << OUT | NOT_HEAD, 4],         // Going right + down corner.

        [LEFT << IN | LEFT << OUT | NOT_HEAD, 6],          // Straight right to left.
        [RIGHT << IN | RIGHT << OUT | NOT_HEAD, 6],        // Straight left to right.
        [UP << IN | UP << OUT | NOT_HEAD, 5],              // Straight bottom to top.
        [DOWN << IN | DOWN << OUT | NOT_HEAD, 5],          // Straight top to bottom.

        [NOT_SET << IN | UP << OUT | NOT_HEAD, 7],         // Going top tail.
        [NOT_SET << IN | DOWN << OUT | NOT_HEAD, 8],       // Going down tail.
        [NOT_SET << IN | LEFT << OUT | NOT_HEAD, 9],       // Going left tail.
        [NOT_SET << IN | RIGHT << OUT | NOT_HEAD, 10],     // Going down tail.

        [DOWN << IN | NOT_SET << OUT | NOT_HEAD, 7],       // Going down half body.
        [UP << IN | NOT_SET << OUT | NOT_HEAD, 8],         // Going top half body.
        [RIGHT << IN | NOT_SET << OUT | NOT_HEAD, 9],      // Going right half body.
        [LEFT << IN | NOT_SET << OUT | NOT_HEAD, 10],      // Going left half body.

        [UP << IN | NOT_SET << OUT | HEAD, 11],            // Going top head.
        [DOWN << IN | NOT_SET << OUT | HEAD, 12],          // Going down head.
        [LEFT << IN | NOT_SET << OUT | HEAD, 13],          // Going left head.
        [RIGHT << IN | NOT_SET << OUT | HEAD, 14],         // Going right head.

        [NOT_SET << IN | NOT_SET << OUT | NOT_HEAD, 15],   // Moignon (last teleported cell).
        [NOT_SET << IN | NOT_SET << OUT | HEAD, 15],       // Moignon (first teleported cell).
    ]);
})();