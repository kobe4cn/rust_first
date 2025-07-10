mod eq;
use std::{
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
#[allow(dead_code)]
pub struct Customer<T> {
    id: u64,
    name: String,
    _type: PhantomData<T>,
}

pub trait Free {
    fn feature1(&self);
    fn feature2(&self);
}

pub trait Personal: Free {
    fn advanced_feature(&self);
}

impl Personal for Customer<PersonalPlan> {
    fn advanced_feature(&self) {
        println!("advanced_feature: {}", self.name);
    }
}

impl<T> Free for Customer<T> {
    fn feature1(&self) {
        println!("feature1: {}", self.name);
    }

    fn feature2(&self) {
        println!("feature2: {}", self.name);
    }
}

impl<T> Customer<T> {
    pub fn new(name: String) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            name,
            _type: PhantomData,
        }
    }
}

impl From<Customer<FreePlan>> for Customer<PersonalPlan> {
    fn from(customer: Customer<FreePlan>) -> Self {
        Self::new(customer.name)
    }
}

pub fn subscribe(customer: Customer<FreePlan>, payment: f32) -> Customer<PersonalPlan> {
    let _plan = PersonalPlan(payment);
    customer.into()
}

#[allow(dead_code)]
pub struct FreePlan;

#[allow(dead_code)]
pub struct PersonalPlan(f32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe() {
        let customer = Customer::<FreePlan>::new("kevin".to_string());
        customer.feature1();
        customer.feature2();
        let customer = subscribe(customer, 10.0);
        customer.advanced_feature();
        customer.feature1();
        customer.feature2();
    }
}
