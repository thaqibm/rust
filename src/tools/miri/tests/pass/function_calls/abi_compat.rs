use std::mem;
use std::num;
use std::ptr;

#[derive(Copy, Clone, Default)]
struct Zst;

#[repr(transparent)]
#[derive(Copy, Clone)]
struct Wrapper<T>(T);

fn id<T>(x: T) -> T {
    x
}

fn test_abi_compat<T: Clone, U: Clone>(t: T, u: U) {
    fn id<T>(x: T) -> T {
        x
    }
    extern "C" fn id_c<T>(x: T) -> T {
        x
    }

    // This checks ABI compatibility both for arguments and return values,
    // in both directions.
    let f: fn(T) -> T = id;
    let f: fn(U) -> U = unsafe { std::mem::transmute(f) };
    let _val = f(u.clone());
    let f: fn(U) -> U = id;
    let f: fn(T) -> T = unsafe { std::mem::transmute(f) };
    let _val = f(t.clone());

    // And then we do the same for `extern "C"`.
    let f: extern "C" fn(T) -> T = id_c;
    let f: extern "C" fn(U) -> U = unsafe { std::mem::transmute(f) };
    let _val = f(u);
    let f: extern "C" fn(U) -> U = id_c;
    let f: extern "C" fn(T) -> T = unsafe { std::mem::transmute(f) };
    let _val = f(t);
}

/// Ensure that `T` is compatible with various repr(transparent) wrappers around `T`.
fn test_abi_newtype<T: Copy + Default>() {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    struct Wrapper2<T>(T, ());
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    struct Wrapper2a<T>((), T);
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    struct Wrapper3<T>(Zst, T, [u8; 0]);

    let t = T::default();
    test_abi_compat(t, Wrapper(t));
    test_abi_compat(t, Wrapper2(t, ()));
    test_abi_compat(t, Wrapper2a((), t));
    test_abi_compat(t, Wrapper3(Zst, t, []));
    test_abi_compat(t, mem::MaybeUninit::new(t)); // MaybeUninit is `repr(transparent)`
}

fn main() {
    // Here we check some of the guaranteed ABI compatibilities.
    // Different integer types of the same size and sign.
    if cfg!(target_pointer_width = "32") {
        test_abi_compat(0usize, 0u32);
        test_abi_compat(0isize, 0i32);
    } else {
        test_abi_compat(0usize, 0u64);
        test_abi_compat(0isize, 0i64);
    }
    test_abi_compat(42u32, num::NonZeroU32::new(1).unwrap());
    // Reference/pointer types with the same pointee.
    test_abi_compat(&0u32, &0u32 as *const u32);
    test_abi_compat(&mut 0u32 as *mut u32, Box::new(0u32));
    test_abi_compat(&(), ptr::NonNull::<()>::dangling());
    // Reference/pointer types with different but sized pointees.
    test_abi_compat(&0u32, &([true; 4], [0u32; 0]));
    // `fn` types
    test_abi_compat(main as fn(), id::<i32> as fn(i32) -> i32);
    // Guaranteed null-pointer-optimizations.
    test_abi_compat(&0u32 as *const u32, Some(&0u32));
    test_abi_compat(main as fn(), Some(main as fn()));
    test_abi_compat(0u32, Some(num::NonZeroU32::new(1).unwrap()));
    test_abi_compat(&0u32 as *const u32, Some(Wrapper(&0u32)));
    test_abi_compat(0u32, Some(Wrapper(num::NonZeroU32::new(1).unwrap())));

    // These must work for *any* type, since we guarantee that `repr(transparent)` is ABI-compatible
    // with the wrapped field.
    test_abi_newtype::<()>();
    test_abi_newtype::<Zst>();
    test_abi_newtype::<u32>();
    test_abi_newtype::<f32>();
    test_abi_newtype::<(u8, u16, f32)>();
    test_abi_newtype::<[u8; 0]>();
    test_abi_newtype::<[u32; 0]>();
    test_abi_newtype::<[u32; 2]>();
    test_abi_newtype::<[u32; 32]>();
}
