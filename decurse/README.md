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
The macros provided by this crate makes your function run on the heap instead.
It works on stable Rust (1.56 at time of writing).

Here's an example to illustrate its mechanism.

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

If we call `factorial(1)`, this is what would happen:
* We run the code in the function starting at point 🅐, as normal.
* When we reach point 🅑, we don't immediately call `factorial(0)`,
instead, we save the information that we have to call `factorial(0)`<sup>1</sup>.
* Once that information is saved, we *pause* the execution of `factorial(1)`, *storing its state on the heap*<sup>2</sup>.
* We then execute `factorial(0)`. At this point the "stack frame" of `factorial(1)` is not on the stack.
It is stored on the heap.
* Once we got the result of `factorial(0)`, we *resume* `factorial(1)` and give it the result of `factorial(0)`.
* The execution continues at point 🅒 and on.

---

<sup>1</sup> To send this information out of the function, we put it in a thread local.

<sup>2</sup> This is accomplished by converting your function into an async function, and awaiting to pause it.
It is somewhat of a hack using async/await.

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

However, it does **not** work on functions with lifetimed types (`&T`, `SomeStruct<'a>`, etc.) in the argument or return type.

### `decurse_unsound`

This macro uses unsafe code in very dangerous ways.
I am far from confident that it is safe, so I'm calling it unsound.
However, I have yet to come up with an example to demonstrate unsoundness,
so there is a small chance that this might actually be sound,
so for brave souls, *try it out*!

This version does not suffer from the limitation of the safe version.
Arguments and return type can be lifetimed just as in any functions.

## Limitations
* As mentioned, the safe variant only works on functions without lifetimed type arguments or lifetimed return type.
	* The [`owning_ref` crate](https://crates.io/crates/owning_ref) is great for working around this.
	* You can use the "unsound" variant, of course. But it might cause problems.
* This is **not** tail-call optimization. Also you can still blow up your heap (although it is much harder).
* One function only. Alternating recursion (`f` calls `g` and `g` calls `f`) is not supported.
Calling the same function but with different generic parameters is not supported.
* Async function is not supported.
* The macro only understand recursive calls that are "direct".

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

## Benchmarks

Benchmarking recursive linear search.
See [the code](https://github.com/wishawa/decurse/tree/main/decurse/examples/benchmark.rs).

| Vec Size 		| Time (normal) (s)		| Time (decurse) (s)	| decurse/normal 		|
|---------------|-----------------------|-----------------------|-----------------------|
| 20000			| 0.19					| 0.63					| 3.37					|
| 40000			| 0.42					| 1.29					| 3.08					|
| 60000			| 0.75					| 1.96					| 2.61					|
| 80000			| 1.20					| 3.04					| 2.54					|
| 100000		| Stack Overflow		| 3.52					| N/A					|
| 120000		| Stack Overflow		| 4.32					| N/A					|
| 140000		| Stack Overflow		| 5.20					| N/A					|
| 160000		| Stack Overflow		| 5.94					| N/A					|
| 180000		| Stack Overflow		| 6.69					| N/A					|

`decurse` version is about 3x **slower** 😦😦😦.

---

Same benchmark with the `slow(8723)` call uncommented for both `linear_search` and `stack_linear_search`.

| Vec Size 		| Time (normal) (s)		| Time (decurse) (s)	| decurse/normal 		|
|---------------|-----------------------|-----------------------|-----------------------|
| 20000			| 0.71					| 2.74					| 0.26					|
| 40000			| 1.23					| 5.30					| 0.23					|
| 60000			| 2.06					| 7.93					| 0.26					|
| 80000			| 2.91					| 10.99					| 0.27					|
| 100000		| Stack Overflow		| 3.57					| N/A					|
| 120000		| Stack Overflow		| 4.56					| N/A					|
| 140000		| Stack Overflow		| 5.08					| N/A					|
| 160000		| Stack Overflow		| 5.72					| N/A					|
| 180000		| Stack Overflow		| 6.75					| N/A					|

`decurse` version is about 4x **faster** 🤔🤔🤔

I expected the `slow()` call to just bring the two versions closer.
It is very strange that the `decurse` version can become faster.
Maybe the stack usage of the normal version makes it harder for CPU to cache things?

---

Anyway, the takeaway here is **do your own benchmark on your own use case**.
The recursive linear search implemented here isn't even something anyone would use!

I would still love to see what the numbers look like for your use cases. Please share!

## Credits
[This blog post by *hurryabit*](https://hurryabit.github.io/blog/stack-safety-for-free/) inspired me to make this.
The main idea is basically the same.
Mine is more hacky because I want to avoid generators (which require nightly and won't be stabilized anytime soon),
so I use async/await instead.
