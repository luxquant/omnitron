#[macro_export]
macro_rules! try_block {
    ($try:block catch ($err:ident : $errtype:ty) $catch:block) => {{
        #[allow(unreachable_code)]
        let result: anyhow::Result<_, $errtype> = (|| Ok::<_, $errtype>($try))();
        match result {
            Ok(_) => (),
            Err($err) => {
                {
                    $catch
                };
            }
        };
    }};
    (async $try:block catch ($err:ident : $errtype:ty) $catch:block) => {{
        let result: anyhow::Result<_, $errtype> = (async { Ok::<_, $errtype>($try) }).await;
        match result {
            Ok(_) => (),
            Err($err) => {
                {
                    $catch
                };
            }
        };
    }};
}

#[test]
#[allow(clippy::assertions_on_constants)]
#[allow(clippy::unwrap_used)]
fn test_catch() {
    let mut caught = false;
    try_block!({
        let _: u32 = "asdf".parse()?;
        assert!(false)
    } catch (e: anyhow::Error) {
        assert_eq!(e.to_string(), "asdf".parse::<i32>().unwrap_err().to_string());
        caught = true;
    });
    assert!(caught);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_success() {
    try_block!({
        let _: u32 = "123".parse()?;
    } catch (_e: anyhow::Error) {
        assert!(false)
    });
}
