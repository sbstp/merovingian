use std::fmt;

pub struct Ident<'t>(pub &'t str);

impl fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.0)
    }
}

pub struct Bind<'t>(pub &'t str);

impl fmt::Display for Bind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ":{}", self.0)
    }
}
