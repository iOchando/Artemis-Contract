//! This contract implements simple counter backed by storage on blockchain.
//!
//! The contract provides methods to [increment] / [decrement] counter and
//! [get it's current value][get_num] or [reset].
//!
//! [increment]: struct.Counter.html#method.increment
//! [decrement]: struct.Counter.html#method.decrement
//! [get_num]: struct.Counter.html#method.get_num
//! [reset]: struct.Counter.html#method.reset

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Balance, Promise};
use near_sdk::collections::{ UnorderedMap};
//use near_sdk::json_types::{U128};
use serde::Serialize;
use serde::Deserialize;
use near_sdk::json_types::{ValidAccountId, U128};
//use near_sdk::env::is_valid_account_id;

near_sdk::setup_alloc!();

pub const VAULT_FEE: u128 = 500;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CoursePurchased {
    course_id: i128,
    pass_certification: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProfileObject {
    user_id: AccountId,
    purchased_courses: Vec<CoursePurchased>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CategoriesObject {
	name: String,
    img: String,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CategoriesJson {
    id: i128,
	name: String,
    img: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateObject {
	title: String,
    description: String,
    content: String,
    tipo: i8, // 1 Video, 2 Text
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateView {
	title: String,
    tipo: i8, // 1 Video, 2 Text
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CoursesObject {
    id: i128,
    creator_id: AccountId,
    title: String,
    categories: CategoriesJson,
    short_description: String,
    long_description: String,
    img: String,
    content: Vec<TemplateObject>,
    price: Balance,
    price_certification: Balance,
    inscriptions: Vec<AccountId>,
    rating: f32,
    reviews: Vec<Review>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Review {
    user_id: AccountId,
    review: String,
    critics: i8,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketView {
    id: i128,
    creator_id: AccountId,
    title: String,
    categories: CategoriesJson,
    short_description: String,
    long_description: String,
    img: String,
    content: Vec<TemplateView>,
    price: Balance,
    price_certification: Balance,
    rating: f32,
    reviews: Vec<Review>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    vault_id: AccountId,
    profiles: Vec<ProfileObject>,
    id_categories: i128,
    categories: Vec<CategoriesJson>,
    id_courses: i128,
    courses: UnorderedMap<i128, CoursesObject>,
    administrators: Vec<AccountId>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default(owner_id: ValidAccountId, vault_id: ValidAccountId) -> Self {
        Self::new(
            owner_id,
            vault_id,
        )
    }

    #[init]
    pub fn new(_owner_id: ValidAccountId, vault_id: ValidAccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            vault_id: vault_id.to_string(),
            profiles: Vec::new(),
            id_categories: 0,
            categories: Vec::new(),
            id_courses: 0,
            courses: UnorderedMap::new(b"s".to_vec()),
            administrators: vec![
                                    "e-learning.testnet".to_string(),
                                    "juanochando.testnet".to_string(),
                                ],
        }
    }

    pub fn set_admin(&mut self, user_id: AccountId) {      
        self.administrators.iter().find(|&x| x == &env::signer_account_id()).expect("Only administrators can set categories");
        let valid = self.administrators.iter().find(|&x| x == &user_id);
        if valid.is_some() {
            env::panic(b"the user is already in the list of administrators");
        }
        self.administrators.push(user_id);
    }

    pub fn delete_admin(&mut self, user_id: AccountId) {      
        self.administrators.iter().find(|&x| x == &env::signer_account_id()).expect("Only administrators can set categories");
        let index = self.administrators.iter().position(|x| x == &user_id.to_string()).expect("the user is not in the list of administrators");
        self.administrators.remove(index);
    }

    pub fn get_profile(&self, user_id: Option<AccountId>) -> Vec<ProfileObject> {
        let mut profiles = self.profiles.clone();

        if user_id.is_some() {
            profiles = self.profiles.iter().filter(|x| x.user_id == user_id.clone().unwrap()).map(|x| ProfileObject {
                user_id: x.user_id.to_string(),
                purchased_courses: x.purchased_courses.clone(),
            }).collect();
        }
        profiles
    }

    pub fn set_category(&mut self, name: String, img: String) -> CategoriesJson {      
        self.administrators.iter().find(|&x| x == &env::signer_account_id()).expect("Only administrators can set categories");
        self.id_categories += 1;
        let data = CategoriesJson {
            id: self.id_categories,
            name: name.to_string(),
            img: img.to_string(),
        };
        
        self.categories.push(data.clone());
        env::log(b"category Created");
        
        data
    }

    pub fn put_category(&mut self, category_id: i128, name: String, img: String) -> CategoriesJson {
        self.administrators.iter().find(|&x| x == &env::signer_account_id()).expect("Only admins can edit categories");
        let index = self.categories.iter().position(|x| x.id == category_id).expect("Category does not exist");
        self.categories[index].name = name.to_string();
        self.categories[index].img = img.to_string();

        env::log(b"Category Update");

        CategoriesJson {
            id: category_id,
            name: name.to_string(),
            img: img.to_string(),
        }
    }

    pub fn get_category(&self, category_id: Option<i128>) -> Vec<CategoriesJson> {
        let mut categories = self.categories.clone();

        if category_id.is_some() {
            categories = self.categories.iter().filter(|x| x.id == category_id.unwrap()).map(|x| CategoriesJson {
                id: x.id,
                name: x.name.to_string(),
                img: x.img.to_string(),
            }).collect();
        }
        categories
    }

    pub fn delete_category(&mut self, category_id: i128) {
        self.administrators.iter().find(|&x| x == &env::signer_account_id()).expect("Only admins can edit categories");
        let index = self.categories.iter().position(|x| x.id == category_id).expect("Category does not exist");
        self.categories.remove(index);

        env::log(b"Category deleted");
    }
    
    pub fn publish_course(&mut self, 
        title: String,
        categories: CategoriesJson,
        short_description: String,
        long_description: String,
        img: String,
        content: Vec<TemplateObject>,
        price: U128,
        price_certification: U128,
    ) -> CoursesObject {
        
        self.id_courses += 1;
        let data = CoursesObject {
            id: self.id_courses,
            creator_id: env::signer_account_id().to_string(),
            title: title.to_string(),
            categories: categories,
            short_description: short_description.to_string(),
            long_description: long_description.to_string(),
            img: img.to_string(),
            content: content,
            price: price.0,
            price_certification: price_certification.0,
            inscriptions: Vec::new(),
            rating: 0.0,
            reviews: Vec::new(),
        };

        self.courses.insert(&self.id_courses, &data);
        env::log(b"published course");
        data
    }

    pub fn put_course(&mut self, 
        course_id: i128,
        title: String,
        categories: CategoriesJson,
        short_description: String,
        long_description: String,
        img: String,
        price: U128,
        price_certification: U128,
    ) -> CoursesObject {
        let course = self.courses.get(&course_id).expect("Course does not exist");

        if course.creator_id == env::signer_account_id().to_string() {
            let data = CoursesObject {
                id: course.id,
                creator_id: course.creator_id,
                title: title,
                categories: categories,
                short_description: short_description.to_string(),
                long_description: long_description.to_string(),
                img: img.to_string(),
                content: course.content,
                price: price.0,
                price_certification: price_certification.0,
                inscriptions: course.inscriptions,
                rating: course.rating,
                reviews: course.reviews,
            };
            self.courses.insert(&course_id, &data);
            env::log(b"updated course");
            data
        } else {
            env::panic(b"No permission")
        }
    }

    pub fn get_courses_intructor(&self, user_id: Option<String>) -> Vec<CoursesObject> {
        if user_id.is_some() {
            self.courses.iter().filter(|(_k, x)| x.creator_id == user_id.clone().unwrap().to_string()).map(|(_k, x)| CoursesObject {
                id: x.id,
                creator_id: x.creator_id.to_string(),
                title: x.title.to_string(),
                categories: x.categories.clone(),
                short_description: x.short_description.to_string(),
                long_description: x.long_description.to_string(),
                img: x.img.to_string(),
                content: x.content.clone(),
                price: x.price,
                price_certification: x.price_certification,
                inscriptions: x.inscriptions.clone(),
                rating: x.rating,
                reviews: x.reviews.clone(),
            }).collect()
        } else {
            env::panic(b"Not user");
        }
    }

    pub fn get_courses_purchased(&self, user_id: String) -> Vec<CoursesObject> {
        let index = self.profiles.iter().position(|x| x.user_id == user_id).expect("Profile does not exist");

        let mut courses_purchased = Vec::new();

        for i in 0..self.profiles[index].purchased_courses.len() {
            courses_purchased.push(self.courses.get(&self.profiles[index].purchased_courses[i].course_id).expect("Artemis: Course does not exists"));
        }

        courses_purchased
    }

    pub fn get_course_id(&self, user_id: String, course_id: i128) -> CoursesObject {
        let index = self.profiles.iter().position(|x| x.user_id == user_id).expect("Profile does not exist");

        self.profiles[index].purchased_courses.iter().position(|k| k.course_id == course_id).expect("Not permission");

        let course = self.courses.get(&course_id).expect("Course does not exist");
        
        course
    }

    pub fn get_pass_certification(&self, user_id: String, course_id: i128) -> CoursePurchased {
        let index = self.profiles.iter().position(|x| x.user_id == user_id).expect("Profile does not exist");

        let index_pass = self.profiles[index].purchased_courses.iter().position(|k| k.course_id == course_id).expect("Not permission");

        self.profiles[index].purchased_courses[index_pass].clone()
    }

    pub fn get_market_courses(&self,
        course_id: Option<i128>,
        creator_id: Option<AccountId>,
        category_id: Option<i128>,
        from_index: Option<u128>,
        limit: Option<u64>
    ) -> Vec<MarketView> {

        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!((self.courses.len() as u128) > start_index, "Out of bounds, please use a smaller from_index.");
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        let mut result: Vec<CoursesObject> = self.courses.iter().map(|(_k, v)| v).collect::<Vec<CoursesObject>>();

        if creator_id.is_some() {
            let creator = creator_id.unwrap().clone();
            result = result.iter().filter(|x| x.creator_id == creator).map(|x| x.clone()).collect();
        };

        if category_id.is_some() {
            let category = category_id.unwrap().clone();
            result = result.iter().filter(|x| x.categories.id == category).map(|x| x.clone()).collect();
        };

        if course_id.is_some() {
            let course = course_id.unwrap().clone();
            result = result.iter().filter(|x| x.id == course).map(|x| x.clone()).collect();
        };

        result.iter()
        .skip(start_index as usize)
        .take(limit)
        .map(|x| MarketView {
            id: x.id,
            creator_id: x.creator_id.to_string(),
            title: x.title.to_string(),
            categories: x.categories.clone(),
            short_description: x.short_description.to_string(),
            long_description: x.long_description.to_string(),
            content: x.content.iter().map(|x| TemplateView {
                title: x.title.to_string(),
                tipo: x.tipo,
            }).collect(),
            img: x.img.to_string(),
            price: x.price,
            price_certification: x.price_certification,
            rating: x.rating,
            reviews: x.reviews.clone(),
        }).collect()
    }

    pub fn get_recent_courses(&self,
        number_courses: u64,
    ) -> Vec<MarketView> {

        if self.courses.len() > number_courses {
            let index: u64 = self.courses.len() - number_courses;
            let result: Vec<CoursesObject> = self.courses.iter().map(|(_k, v)| v).collect::<Vec<CoursesObject>>();

            result.iter()
            .skip(index as usize)
            .map(|x| MarketView {
                id: x.id,
                creator_id: x.creator_id.to_string(),
                title: x.title.to_string(),
                categories: x.categories.clone(),
                short_description: x.short_description.to_string(),
                long_description: x.long_description.to_string(),
                img: x.img.to_string(),
                content: x.content.iter().map(|x| TemplateView {
                    title: x.title.to_string(),
                    tipo: x.tipo,
                }).collect(),
                price: x.price,
                price_certification: x.price_certification,
                rating: x.rating,
                reviews: x.reviews.clone(),
            }).collect()
        } else {
            self.courses.iter().map(|(_k, x)| MarketView {
                id: x.id,
                creator_id: x.creator_id.to_string(),
                title: x.title.to_string(),
                categories: x.categories.clone(),
                short_description: x.short_description.to_string(),
                long_description: x.long_description.to_string(),
                img: x.img.to_string(),
                content: x.content.iter().map(|x| TemplateView {
                    title: x.title.to_string(),
                    tipo: x.tipo,
                }).collect(),
                price: x.price,
                price_certification: x.price_certification,
                rating: x.rating,
                reviews: x.reviews.clone(),
            }).collect()
        }  
    }

    pub fn delete_course(&mut self, course_id: i128) {
        let course = self.courses.get(&course_id).expect("Course does not exist");

        if course.creator_id == env::signer_account_id().to_string() {
            if course.inscriptions.len() == 0 {
                self.courses.remove(&course_id);
                env::log(b"Course deleted")
            } else {
                env::panic(b"Can't delete course")
            }
        } else {
            env::panic(b"No permission")
        }
    }

    pub fn get_course_size(&self,
        creator_id: Option<AccountId>,
        category_id: Option<i128>,) -> u64 {
        let mut result: Vec<CoursesObject> = self.courses.iter().map(|(_k, v)| v).collect::<Vec<CoursesObject>>();

        if creator_id.is_some() {
            let creator = creator_id.unwrap().clone();
            result = result.iter().filter(|x| x.creator_id == creator).map(|x| x.clone()).collect();
        };

        if category_id.is_some() {
            let category = category_id.unwrap().clone();
            result = result.iter().filter(|x| x.categories.id == category).map(|x| x.clone()).collect();
        };

        result.len().try_into().unwrap()
    }

    #[payable]
    pub fn course_buy(
        &mut self, 
        course_id: i128, 
    ) -> CoursesObject {
        let initial_storage_usage = env::storage_usage();

        let mut course = self.courses.get(&course_id).expect("Artemis: Course does not exist");

        let index = course.inscriptions.iter().position(|x| x.to_string() == env::signer_account_id().to_string());

        if index.is_some() {
            env::panic(b"Artemis: User already enrolled in the course");
        }

        let price: Balance = course.price;
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= price,
            "Artemis: attached deposit is less than price : {}",
            price
        );

        let for_vault = price as u128 * VAULT_FEE / 10_000u128;
        let price_deducted = price - for_vault;
        Promise::new(course.creator_id.to_string()).transfer(price_deducted);

        if for_vault != 0 {
            Promise::new(self.vault_id.clone()).transfer(for_vault);
        }

        refund_deposit(env::storage_usage() - initial_storage_usage, price);

        course.inscriptions.push(env::signer_account_id().to_string());
        self.courses.insert(&course_id, &course);

        self.profile_inscription(course_id);

        course
    }

    #[payable]
    pub fn pass_certification_buy(
        &mut self, 
        course_id: i128, 
    ) -> CoursePurchased {
        let initial_storage_usage = env::storage_usage();

        let course = self.courses.get(&course_id).expect("Artemis: Course does not exist");

        let index = self.profiles.iter().position(|x| x.user_id == env::signer_account_id()).expect("Profile does not exist");

        let price_certification: Balance = course.price_certification;
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= price_certification,
            "Artemis: attached deposit is less than price : {}",
            price_certification
        );

        let for_vault = price_certification as u128 * VAULT_FEE / 10_000u128;
        let price_deducted = price_certification - for_vault;
        Promise::new(course.creator_id.to_string()).transfer(price_deducted);

        if for_vault != 0 {
            Promise::new(self.vault_id.clone()).transfer(for_vault);
        }

        refund_deposit(env::storage_usage() - initial_storage_usage, price_certification);

        let index_course = self.profiles[index].purchased_courses.iter().position(|k| k.course_id == course_id).expect("Course does not buy");
        self.profiles[index].purchased_courses[index_course].pass_certification = true;

        self.profiles[index].purchased_courses[index_course].clone()
    }

    pub fn change_pass_certification(&mut self, user_id: AccountId, course_id: i128,) -> CoursePurchased {      
        self.administrators.iter().find(|&x| x == &env::signer_account_id()).expect("Only administrators can set categories");
        
        let index = self.profiles.iter().position(|x| x.user_id == user_id.to_string()).expect("Profile does not exist");

        let index_course = self.profiles[index].purchased_courses.iter().position(|k| k.course_id == course_id).expect("Course does not buy");

        self.profiles[index].purchased_courses[index_course].pass_certification = false;

        self.profiles[index].purchased_courses[index_course].clone()
    }

    pub fn set_review(
        &mut self, 
        course_id: i128, 
        review: String,
        critics: i8,
    ) -> Review {

        let mut course = self.courses.get(&course_id).expect("Artemis: Course does not exist");

        let index = course.reviews.iter().position(|x| x.user_id == env::signer_account_id().to_string());

        let data = Review {
            user_id: env::signer_account_id().to_string(),
            review: review.to_string(),
            critics: critics,
        };

        if index.is_some() {
            let ind = course.reviews.iter().position(|x| x.user_id == env::signer_account_id().to_string()).expect("Artemis: Review does not exist");
            course.reviews[ind] = data.clone();
            let mut cont = 0.0;
            for item in &course.reviews {
                cont += item.critics as f32;
            }
            course.rating = cont / (course.reviews.len() as f32);
            self.courses.insert(&course_id, &course);
            return data
        }

        course.reviews.push(data.clone());
        let mut cont = 0.0;
        for item in &course.reviews {
            cont += item.critics as f32;
        }
        course.rating = cont / (course.reviews.len() as f32);

        self.courses.insert(&course_id, &course);

        data
    }

    pub fn get_review(&self,
        course_id: i128, 
        user_id: AccountId
    ) -> Vec<Review> {
        let mut result: Vec<CoursesObject> = self.courses.iter().map(|(_k, v)| v).collect::<Vec<CoursesObject>>();

        result = result.iter().filter(|x| x.id == course_id).map(|x| x.clone()).collect();

        let review: Vec<Review> = result[0].reviews.iter().filter(|x| x.user_id == user_id).map(|x| x.clone()).collect();

        review
    }

    fn profile_inscription(&mut self, course_id: i128) {
        let indexaux = self.profiles.iter().position(|x| x.user_id == env::signer_account_id());//.expect("Category does not exist");

        if indexaux.is_some() {
            let index = self.profiles.iter().position(|x| x.user_id == env::signer_account_id()).expect("Profile does not exist");
            self.profiles[index].user_id = env::signer_account_id().to_string();
            let course = CoursePurchased {
                course_id: course_id,
                pass_certification: false,
            };
            self.profiles[index].purchased_courses.push(course);
        } else {
            let course = CoursePurchased {
                course_id: course_id,
                pass_certification: false,
            };
            let data = ProfileObject {
                user_id: env::signer_account_id().to_string(),
                purchased_courses: vec![course],
            };
            
            self.profiles.push(data.clone());
            env::log(b"profile and course purchased Created");
        }
    }

    pub fn get_courses_rating(&self, top: Option<i32>) -> Vec<MarketView> {
        let top_limit = top.unwrap_or(12);

        let mut top_courses: Vec<CoursesObject> = self.courses.iter()
                                                .filter(|(_k, v)| v.rating > 0.0)
                                                .map(|(_k, v)| v).collect::<Vec<CoursesObject>>();

        top_courses.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
        
        top_courses.iter()
        .take(top_limit as usize)
        .map(|x| MarketView {
            id: x.id,
            creator_id: x.creator_id.to_string(),
            title: x.title.to_string(),
            categories: x.categories.clone(),
            short_description: x.short_description.to_string(),
            long_description: x.long_description.to_string(),
            content: x.content.iter().map(|x| TemplateView {
                title: x.title.to_string(),
                tipo: x.tipo,
            }).collect(),
            img: x.img.to_string(),
            price: x.price,
            price_certification: x.price_certification,
            rating: x.rating,
            reviews: x.reviews.clone(),
        }).collect()
    }

}

fn refund_deposit(storage_used: u64, extra_spend: Balance) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit() - extra_spend;

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

// unlike the struct's functions above, this function cannot use attributes #[derive(…)] or #[near_bindgen]
// any attempts will throw helpful warnings upon 'cargo build'
// while this function cannot be invoked directly on the blockchain, it can be called from an invoked function

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-counter-tutorial -- --nocapture
 * Note: 'rust-counter-tutorial' comes from cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // part of writing unit tests is setting up a mock context
    // in this example, this is only needed for env::log in the contract
    // this is also a useful list to peek at when wondering what's available in env::*
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "robert.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "jane.testnet".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    // mark individual unit tests with #[test] for them to be registered and fired
    #[test]
    fn increment() {
        // set up the mock context into the testing environment
        let context = get_context(vec![], false);
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract = Counter { val: 0 };
        contract.increment();
        println!("Value after increment: {}", contract.get_num());
        // confirm that we received 1 when calling get_num
        assert_eq!(1, contract.get_num());
    }

    #[test]
    fn decrement() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Counter { val: 0 };
        contract.decrement();
        println!("Value after decrement: {}", contract.get_num());
        // confirm that we received -1 when calling get_num
        assert_eq!(-1, contract.get_num());
    }

    #[test]
    fn increment_and_reset() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Counter { val: 0 };
        contract.increment();
        contract.reset();
        println!("Value after reset: {}", contract.get_num());
        // confirm that we received -1 when calling get_num
        assert_eq!(0, contract.get_num());
    }
}