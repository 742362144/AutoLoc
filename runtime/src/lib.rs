#![feature(generators, generator_trait)]
#![feature(llvm_asm)]

pub mod task;
pub mod executor;
pub mod policy;
pub mod invoke;
pub mod cycles;
pub mod ext;
mod sys;

pub const PARALLEL: i32 = 128;
pub const LIMIT: u64 = 6;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
