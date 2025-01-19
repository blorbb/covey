/**
 * This needs to be a thrown exception instead of a function that returns never
 * as typescript infers the return type as undefined.
 */
export class UnreachableError extends Error {
    constructor(x: never) {
        super(`unreachable reached: ${x}`)
    }
}