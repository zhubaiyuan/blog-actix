use crate::errors::AppError;
use crate::schema::users;
use diesel::prelude::*;

type Result<T> = std::result::Result<T, AppError>;
