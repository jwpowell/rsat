pub enum Error {
    WidthMismatch,
}

pub trait Logic {
    type Expr;

    fn width(&self, a: &Self::Expr) -> usize;

    fn bit(&mut self, name: &str) -> Self::Expr;
    fn bit_unnamed(&mut self) -> Self::Expr;

    fn word(&mut self, name: &str, width: usize) -> Self::Expr;
    fn word_unnamed(&mut self, width: usize) -> Self::Expr;

    fn and(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn or(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn xor(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn not(&mut self, a: &Self::Expr) -> Result<Self::Expr, Error>;

    fn implies(&mut self, pred: &Self::Expr, concl: &Self::Expr) -> Result<Self::Expr, Error>;

    fn if_then_else(
        &mut self,
        test: &Self::Expr,
        yes: &Self::Expr,
        no: &Self::Expr,
    ) -> Result<Self::Expr, Error>;

    fn shl(&mut self, a: &Self::Expr, k: usize) -> Result<Self::Expr, Error>;
    fn shl_by_var(&mut self, a: &Self::Expr, k: &Self::Expr) -> Result<Self::Expr, Error>;

    fn shr(&mut self, a: &Self::Expr, k: usize) -> Result<Self::Expr, Error>;
    fn shr_by_var(&mut self, a: &Self::Expr, k: &Self::Expr) -> Result<Self::Expr, Error>;

    fn rotl(&mut self, a: &Self::Expr, k: usize) -> Result<Self::Expr, Error>;
    fn rotl_by_var(&mut self, a: &Self::Expr, k: &Self::Expr) -> Result<Self::Expr, Error>;

    fn rotr(&mut self, a: &Self::Expr, k: usize) -> Result<Self::Expr, Error>;
    fn rotr_by_var(&mut self, a: &Self::Expr, k: &Self::Expr) -> Result<Self::Expr, Error>;

    fn test_lt(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn test_le(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn test_gt(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn test_ge(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn test_eq(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn test_ne(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;

    fn slice(&mut self, a: &Self::Expr, lo: usize, hi: usize) -> Result<Self::Expr, Error>;
    fn concat(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;

    fn add(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn sub(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn mul(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn div(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;
    fn rem(&mut self, a: &Self::Expr, b: &Self::Expr) -> Result<Self::Expr, Error>;

    fn gc(&mut self, live: &[Self::Expr]);
    fn simplify(&mut self);
}
