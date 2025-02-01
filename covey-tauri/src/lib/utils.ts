// https://stackoverflow.com/a/49670389
export type DeepReadonly<T> =
  // array
  T extends (infer R)[]
    ? DeepReadonlyArray<R>
    : // function
      T extends Function
      ? T
      : // object
        T extends object
        ? DeepReadonlyObject<T>
        : // other
          T;

type DeepReadonlyArray<T> = ReadonlyArray<DeepReadonly<T>>;

type DeepReadonlyObject<T> = {
  readonly [P in keyof T]: DeepReadonly<T[P]>;
};

export class UnreachableError extends Error {
  constructor(x: never) {
    // eslint-disable-next-line @typescript-eslint/restrict-template-expressions -- in case of incorrect type
    super(`unreachable reached: ${x}`);
  }
}

/**
 * Asserts that a value is never at compile time.
 *
 * Throws an `UnreachableError` if this is reached.
 */
export const unreachable = (x: never): never => {
  throw new UnreachableError(x);
};
