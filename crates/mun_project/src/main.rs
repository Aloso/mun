use std::path::Path;
use std::thread;
use std::time::Duration;

use mun_runtime::MunRuntime;

fn main() {
    let mut runtime =
        MunRuntime::new(Duration::from_millis(10)).expect("Failed to initialize Mun runtime.");

    let manifest_path = Path::new("..\\mun_test\\Cargo.toml");

    runtime
        .add_manifest(&manifest_path)
        .expect("Failed to load shared library.");

    loop {
        runtime.update();

        runtime.invoke_library_method::<()>(&manifest_path, "load", &[]);
        runtime.invoke_library_method::<()>(&manifest_path, "init", &[]);

        let a: f32 = 4.0;
        let b: f32 = 2.0;
        let c: f32 = runtime.invoke_library_method(&manifest_path, "add", &[&a, &b]);

        println!("{a} + {b} = {c}", a = a, b = b, c = c);

        runtime.invoke_library_method::<()>(&manifest_path, "deinit", &[]);
        runtime.invoke_library_method::<()>(&manifest_path, "unload", &[]);

        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use mun_runtime::MunRuntime;
    use std::path::Path;
    use std::time::Duration;

    #[test]
    fn mun_invoke_library_method() {
        let mut runtime =
            MunRuntime::new(Duration::from_millis(10)).expect("Failed to initialize Mun runtime.");

        let manifest_path = Path::new("..\\mun_test\\Cargo.toml");

        runtime
            .add_manifest(&manifest_path)
            .expect("Failed to load shared library.");

        let a: f32 = 4.0;
        let b: f32 = 2.0;

        assert_eq!(
            runtime.invoke_library_method::<f32>(&manifest_path, "add", &[&a, &b]),
            a + b
        );
    }
}
