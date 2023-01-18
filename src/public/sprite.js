const FRAMES_MAPPING = (() => {
    const HEAD = 1;
    const NOT_HEAD = NOT_SET = 0;
    const UP = 9;                           // 0b0100_1
    const DOWN = 17;                        // 0b1000_1
    const LEFT = 3;                         // 0b0001_1
    const RIGHT = 5;                        // 0b0010_1

    return new Map([
        [DOWN, RIGHT, NOT_HEAD, 1],         // Going down + right corner.
        [LEFT, UP, NOT_HEAD, 1],            // Going left + up corner.
        [DOWN, LEFT, NOT_HEAD, 2],          // Going down + left corner.
        [RIGHT, UP, NOT_HEAD, 2],           // Going right + up corner.
        [UP, RIGHT, NOT_HEAD, 3],           // Going up + right corner.
        [LEFT, DOWN, NOT_HEAD, 3],          // Going left + down corner.
        [UP, LEFT, NOT_HEAD, 4],            // Going up + left corner.
        [RIGHT, DOWN, NOT_HEAD, 4],         // Going right + down corner.

        [LEFT, LEFT, NOT_HEAD, 6],          // Straight right to left.
        [RIGHT, RIGHT, NOT_HEAD, 6],        // Straight left to right.
        [UP, UP, NOT_HEAD, 5],              // Straight bottom to top.
        [DOWN, DOWN, NOT_HEAD, 5],          // Straight top to bottom.

        [NOT_SET, UP, NOT_HEAD, 7],         // Going top tail.
        [NOT_SET, DOWN, NOT_HEAD, 8],       // Going down tail.
        [NOT_SET, LEFT, NOT_HEAD, 9],       // Going left tail.
        [NOT_SET, RIGHT, NOT_HEAD, 10],     // Going down tail.

        [DOWN, NOT_SET, NOT_HEAD, 7],       // Going down half body.
        [UP, NOT_SET, NOT_HEAD, 8],         // Going top half body.
        [RIGHT, NOT_SET, NOT_HEAD, 9],      // Going right half body.
        [LEFT, NOT_SET, NOT_HEAD, 10],      // Going left half body.

        [UP, NOT_SET, HEAD, 11],            // Going top head.
        [DOWN, NOT_SET, HEAD, 12],          // Going down head.
        [LEFT, NOT_SET, HEAD, 13],          // Going left head.
        [RIGHT, NOT_SET, HEAD, 14],         // Going right head.

        [NOT_SET, NOT_SET, NOT_HEAD, 15],   // Moignon (last teleported cell).
        [NOT_SET, NOT_SET, HEAD, 15],       // Moignon (first teleported cell).
    ].map(([from, to, head, frame]) => [from << 6 | to << 1 | head, frame]));
})();