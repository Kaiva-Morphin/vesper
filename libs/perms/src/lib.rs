pub mod user;
pub mod groups;

#[macro_export]
macro_rules! rule {
    ($const_name:ident, $value:expr) => {
        pub const $const_name: &str = $value;

        paste::paste! {
            pub fn [<$const_name:lower>](key: &str) -> String {
                format!("{value}.{key}", value = $value)
            }
            pub fn [<$const_name:lower _postfix>](key: &str, postfix: &str) -> String {
                format!("{value}.{key}.{postfix}", value = $value, postfix = postfix)
            }
        }
    };
}
