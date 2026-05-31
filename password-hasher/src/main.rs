use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use inquire::Password;

fn main() {
    let pwd = Password::new("Password:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()
        .unwrap();
    let password = pwd.as_bytes();

    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();
    println!("{}", password_hash);
}
