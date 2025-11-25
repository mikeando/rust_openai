use std::marker::PhantomData;

use rand::Rng;

pub trait Generatable {
    fn gen(context: &mut GeneratorContext) -> Self;
}

impl Generatable for String {
    fn gen(context: &mut GeneratorContext) -> Self {
        let id: u64 = context.rng.gen();
        format!("String{:016x}", id)
    }
}

impl<T> Generatable for Option<T>
where
    T: Generatable,
{
    fn gen(context: &mut GeneratorContext) -> Self {
        match context.rng.gen_bool(0.25) {
            true => None,
            false => Some(context.gen()),
        }
    }
}

pub struct GeneratorContext {
    pub rng: rand::rngs::ThreadRng,
}

impl GeneratorContext {
    pub fn new() -> GeneratorContext {
        let rng = rand::thread_rng();
        GeneratorContext { rng }
    }

    pub fn gen<T>(&mut self) -> T
    where
        T: Generatable,
    {
        T::gen(self)
    }
}

impl Default for GeneratorContext {
    fn default() -> Self {
        Self::new()
    }
}
pub trait Generator<T> {
    fn gen(&self, context: &mut GeneratorContext) -> T;
}

pub struct DefaultGenerator<T: Generatable> {
    _t: PhantomData<T>,
}

impl<T: Generatable> DefaultGenerator<T> {
    fn new() -> DefaultGenerator<T> {
        DefaultGenerator { _t: PhantomData }
    }
}

impl<T: Generatable> Generator<T> for DefaultGenerator<T> {
    fn gen(&self, context: &mut GeneratorContext) -> T {
        T::gen(context)
    }
}

pub struct FuncGenerator<T, F>
where
    F: Fn(&mut GeneratorContext) -> T,
{
    _t: PhantomData<T>,
    f: F,
}

impl<T, F> Generator<T> for FuncGenerator<T, F>
where
    F: Fn(&mut GeneratorContext) -> T,
{
    fn gen(&self, context: &mut GeneratorContext) -> T {
        (self.f)(context)
    }
}

pub struct VectorGenerator<T, G: Generator<T>> {
    _t: PhantomData<T>,
    g: G,
    min: usize,
    max: usize,
}

impl<T, G: Generator<T>> Generator<Vec<T>> for VectorGenerator<T, G> {
    fn gen(&self, context: &mut GeneratorContext) -> Vec<T> {
        let n = context.rng.gen_range(self.min..=self.max);
        (0..n).map(|_| self.g.gen(context)).collect()
    }
}

pub struct OptionGenerator<T, G: Generator<T>> {
    _t: PhantomData<T>,
    g: G,
    p: f64,
}

impl<T, G: Generator<T>> Generator<Option<T>> for OptionGenerator<T, G> {
    fn gen(&self, context: &mut GeneratorContext) -> Option<T> {
        if context.rng.gen_bool(self.p) {
            Some(self.g.gen(context))
        } else {
            None
        }
    }
}

pub fn vec_gen<T, G: Generator<T>>(min: usize, max: usize, g: G) -> VectorGenerator<T, G> {
    VectorGenerator {
        _t: PhantomData,
        g,
        min,
        max,
    }
}

pub fn opt_gen<T, G: Generator<T>>(p: f64, g: G) -> OptionGenerator<T, G> {
    OptionGenerator {
        _t: PhantomData,
        g,
        p,
    }
}

pub fn func_gen<T, F>(f: F) -> FuncGenerator<T, F>
where
    F: Fn(&mut GeneratorContext) -> T,
{
    FuncGenerator { _t: PhantomData, f }
}

pub fn gen_vec<T: Generatable>(context: &mut GeneratorContext, min: usize, max: usize) -> Vec<T> {
    vec_gen(min, max, DefaultGenerator::<T>::new()).gen(context)
}

pub fn gen_opt_vec<T: Generatable>(
    context: &mut GeneratorContext,
    p: f64,
    min: usize,
    max: usize,
) -> Option<Vec<T>> {
    let g = DefaultGenerator::<T>::new();
    let vg = vec_gen(min, max, g);
    opt_gen(p, vg).gen(context)
}

pub fn gen_opt_x<F, T>(context: &mut GeneratorContext, p: f64, f: F) -> Option<T>
where
    F: Fn(&mut GeneratorContext) -> T,
{
    if !context.rng.gen_bool(p) {
        return None;
    }
    Some(f(context))
}

pub fn gen_opt<T: Generatable>(context: &mut GeneratorContext, p: f64) -> Option<T> {
    gen_opt_x(context, p, T::gen)
}
