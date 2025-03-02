
pub trait ReplaceErr<T, E> {
    fn replace_err<NE>(self, new_err: NE) -> Result<T, NE>;
}

impl<T, E> ReplaceErr<T, E> for Result<T, E> {
    fn replace_err<NE>(self, new_err: NE) -> Result<T, NE> {
        match self {
            Ok(v) => {Ok(v)}
            Err(_) => {Err(new_err)}
        }
    }
}