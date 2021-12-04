# Decurse

[<img alt="crates.io" src="https://img.shields.io/crates/v/decurse?style=for-the-badge" height="20">](https://crates.io/crates/decurse)
[<img alt="crates.io" src="https://img.shields.io/docsrs/decurse?style=for-the-badge" height="20">](https://docs.rs/decurse)

## Example

```rust
#[decurse::decurse] // 👈 Slap this on your recursive function and stop worrying about stack overflow!
fn factorial(x: u32) -> u32 {
	if x == 0 {
		1
	} else {
		x * factorial(x - 1)
	}
}

println!("{}", factorial(10));
```
More examples (fibonacci, DFS, ...) are in the [examples directory](https://github.com/wishawa/decurse/tree/main/decurse/examples/).

## Functionality
The macros provided by this crate make your recursive functions run on the heap instead.
Works on stable Rust (1.56 at the time of writing).

Here's an example to illustrate the mechanism.

```rust
fn factorial(x: u32) -> u32 {
	// 🅐
	if x == 0 {
		1
	} else {
		
		let rec = {
			// 🅑
			factorial(x - 1)
		};

		// 🅒
		rec * x
	}
}
```

If we call `factorial(1)`, the following would happen:
* We run the code in the function starting at point 🅐.
* When we reach point 🅑, we don't immediately call `factorial(0)`,
instead, we save the information that we have to call `factorial(0)`<sup>1</sup>.
* Once that information is saved, we *pause* the execution of `factorial(1)`, *storing the state on the heap*<sup>2</sup>.
* We then execute `factorial(0)`. During this, the "stack state" of `factorial(1)` is not on the stack.
It is stored on the heap.
* Once we got the result of `factorial(0)`, we *resume* `factorial(1)` giving it the result of `factorial(0)`<sup>3</sup>.
* The execution continues at point 🅒 and on.

---

<sup>1</sup> To send this information out of the function, we put it in a thread local.

<sup>2</sup> This is accomplished by converting your function into an async function, and awaiting to pause it.
It is somewhat of a hack using async/await.

<sup>3</sup> This again use thread local.

---

<details>
<summary>Click to show an example of what the macro expands to</summary>

```rust
fn factorial(arg_0: u32) -> u32 {
	async fn factorial(x: u32) -> u32 {
		if x == 0 {
			1
		} else {
			x * ({
				// Save what we have to do next.
				::decurse::for_macro_only::sound::set_next(factorial(x - 1));
				// Pause the current function.
				::decurse::for_macro_only::sound::PendOnce::new().await;
				// Once resumed, get the result.
				::decurse::for_macro_only::sound::get_result(factorial)
			})
		}
	}
	::decurse::for_macro_only::sound::execute(factorial(arg_0))
}
```

</details>

## Usage

This crate provides two macros: `decurse` and `decurse_unsound`.
Simply put them on top of your function.

```rust
#[decurse::decurse]
fn some_function(...) -> ...
```

```rust
#[decurse::decurse_unsound]
fn some_function(...) -> ...
```

### `decurse`

This is the version you should prefer. 
This does not use unsafe code and is thus **safe**.

However, it does **not** work on functions with lifetimed types (`&T`, `SomeStruct<'a>`, etc.) in the argument.

### `decurse_unsound`

This macro can cause unsoundness (see [example](https://github.com/wishawa/decurse/blob/main/decurse/examples/unsound_usage.rs)).
My (unproven) believe is that if a function compiles without `#[decurse_unsound]`, then putting `#[decurse_unsound]` on it should be safe.

This version does not suffer from the limitation of the safe version.
Arguments can be lifetimed just as in any functions.

## Limitations
* As mentioned, the safe variant only works on functions without lifetimed type arguments.
	* The [`owning_ref` crate](https://crates.io/crates/owning_ref) is great for working around this.
	* You can use the "unsound" variant, of course. But it might cause problems.
* This is **not** tail-call optimization. Also you can still blow up your heap (although it is much harder).
* One function only. Alternating recursion (`f` calls `g` then `g` calls `f`) is not supported.
Calling the same function but with different generic parameters is not supported.
* Async function are not supported.
* Struct methods are not supported. Freestanding function only.
* The macro only understand recursive calls that are written literally.

	```rust
	// This would work:
	recursive(x - 1);

	// The macro wouldn't understand this:
	let f = recursive;
	f(x - 1);
	```

* The function must have no more than 12 arguments.
	* This is actually a limitation of the [`pfn` crate](https://crates.io/crates/pfn).
* `impl Trait` in argument position is not supported.
	* You can use normal, named, generics.
* This is still very experimental. The safe variant doesn't contain unsafe code but even then you should still be careful.
* Multithreading is not supported.

## Benchmarks

Benchmarking recursive linear search.
See [the code](https://github.com/wishawa/decurse/tree/main/decurse/examples/benchmark.rs).

| Vec Size 		| Time (decurse) (s)	| Time (normal) (s)		| decurse/normal 		|
|---------------|-----------------------|-----------------------|-----------------------|
| 20000			| 0.65					| 0.19					| 3.45					|
| 40000			| 1.29					| 0.43					| 2.96					|
| 60000			| 2.11					| 0.78					| 2.69					|
| 80000			| 2.81					| 1.24					| 2.27					|
| 100000		| 3.49					| Stack Overflow		| N/A					|
| 120000		| 4.32					| Stack Overflow		| N/A					|
| 140000		| 5.23					| Stack Overflow		| N/A					|
| 160000		| 5.99					| Stack Overflow		| N/A					|
| 180000		| 6.72					| Stack Overflow		| N/A					|

`decurse` version runs at about 35% the performance of the normal version.

---

Same benchmark with the `slow(8723)` call uncommented for both `linear_search` and `stack_linear_search`.
`slow()` is an artificial computation to mimick real use cases where the recursive function actually does something.

| Vec Size 		| Time (decurse) (s)	| Time (normal) (s)		| decurse/normal 		|
|---------------|-----------------------|-----------------------|-----------------------|
| 20000			| 2.87					| 2.56					| 1.12					|
| 40000			| 5.74					| 5.18					| 1.11					|
| 60000			| 8.64					| 7.80					| 1.11					|
| 80000			| 11.57					| 10.49					| 1.10					|
| 100000		| 14.59					| Stack Overflow		| N/A					|
| 120000		| 17.60					| Stack Overflow		| N/A					|
| 140000		| 20.59					| Stack Overflow		| N/A					|
| 160000		| 23.60					| Stack Overflow		| N/A					|
| 180000		| 26.61					| Stack Overflow		| N/A					|

`decurse` version runs at about 90% the performance of the normal version.

---

Anyway, you should **do your own benchmarks for your own use cases**.
The recursive linear search implemented here isn't even something anyone would use!

I would still love to see what the numbers look like for your use cases. Please share!

## Credits
[This blog post by *hurryabit*](https://hurryabit.github.io/blog/stack-safety-for-free/) inspired me to make this.
The main idea is basically the same.
Mine is more hacky because I want to avoid generators (which require nightly and won't be stabilized anytime soon),
so I use async/await instead.
