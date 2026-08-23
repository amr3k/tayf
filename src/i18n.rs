pub fn set_app_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}

pub fn current_app_locale() -> String {
    rust_i18n::locale().to_string()
}

pub fn is_rtl() -> bool {
    let loc = current_app_locale();
    loc.starts_with("ar")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_switching_and_rtl() {
        set_app_locale("en");
        assert_eq!(current_app_locale(), "en");
        assert!(!is_rtl());

        set_app_locale("ar");
        assert_eq!(current_app_locale(), "ar");
        assert!(is_rtl());

        set_app_locale("en");
    }
}
