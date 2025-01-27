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
