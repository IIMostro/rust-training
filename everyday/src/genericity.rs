use anyhow::Result;

pub trait Calculate{
    fn ufunc(&self, left: usize, right: usize) -> Result<usize>;
}

#[derive(Default, Clone)]
pub struct Add;

impl Calculate for Add {
    fn ufunc(&self, left: usize, right: usize) -> Result<usize> {
        Ok(left + right)
    }
}

#[derive(Default, Clone)]
pub struct Sub;

impl Calculate for Sub {
    fn ufunc(&self, left: usize, right: usize) -> Result<usize> {
        Ok(left - right)
    }
}

pub struct Algorithm {
    pub left: usize,
    pub right: usize,
    pub algorithm: Option<Box<dyn Calculate>>
}

impl Algorithm {

    pub fn execute(self) -> Result<usize>{
        let algorithm = self.algorithm.unwrap();
        algorithm.ufunc(self.left, self.right)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use crate::genericity::{Add, Algorithm};

    #[test]
    pub fn test_genericity_should_work() -> Result<()> {
        let algorithm = Algorithm{
            left: 1,
            right: 2,
            algorithm: Some(Box::new(Add::default()))
        };
        let result = algorithm.execute();
        println!("{}", result.unwrap());
        Ok(())
    }
}