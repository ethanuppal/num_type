# num_type

Automatically derive `num_trait` traits for your newtype wrapper.

For example, the following types will have `From`, `Into`, `Add`, `Sub`, `Zero`,
etc. implemented for them, so you can add them with, e.g., `+`.

```rs
#[num_type]
struct Wrapper(i32);

#[num_type]
struct Wrapper2(u64);
```

```
```
