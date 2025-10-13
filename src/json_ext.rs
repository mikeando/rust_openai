/// Extensions for serde_json::Value to make some of the things we need to do repeatedly easier.
use crate::types::Error;

pub trait JsonValueExt {
    fn map_array<F, T>(&self, f: F) -> Result<Vec<T>, Error>
    where
        F: FnMut(&Self) -> T;

    fn flat_map_array<F, T>(&self, f: F) -> Result<Vec<T>, Error>
    where
        F: FnMut(&Self) -> Result<T, Error>;

    fn flat_map_opt_array<F, T>(&self, f: F) -> Result<Option<Vec<T>>, Error>
    where
        F: FnMut(&Self) -> Result<T, Error>;

    fn map_opt_obj<F, T>(&self, f: F) -> Result<Option<T>, Error>
    where
        F: FnMut(&Self) -> Result<T, Error>;

    fn map_opt<F, T>(&self, f: F) -> Result<Option<T>, Error>
    where
        F: FnMut(&Self) -> Result<T, Error>;

    fn to_opt_u32(&self) -> Result<Option<u32>, Error>;
    fn to_opt_f32(&self) -> Result<Option<f32>, Error>;
    fn to_opt_i32(&self) -> Result<Option<i32>, Error>;
    fn to_opt_string(&self) -> Result<Option<String>, Error>;
    fn to_opt_bool(&self) -> Result<Option<bool>, Error>;
    fn to_string_or_err(&self) -> Result<String, Error>;
}

impl JsonValueExt for serde_json::Value {
    fn map_array<F, T>(&self, f: F) -> Result<Vec<T>, Error>
    where
        F: FnMut(&Self) -> T,
    {
        // First check it's an array
        let array = self.as_array().ok_or(Error::JsonExpectedArray)?;
        Ok(array.iter().map(f).collect())
    }

    fn flat_map_array<F, T>(&self, f: F) -> Result<Vec<T>, Error>
    where
        F: FnMut(&Self) -> Result<T, Error>,
    {
        // First check it's an array
        let array = self.as_array().ok_or(Error::JsonExpectedArray)?;
        array.iter().map(f).collect()
    }

    fn flat_map_opt_array<F, T>(&self, f: F) -> Result<Option<Vec<T>>, Error>
    where
        F: FnMut(&Self) -> Result<T, Error>,
    {
        if self.is_null() {
            return Ok(None);
        }
        self.flat_map_array(f).map(Some)
    }

    fn map_opt_obj<F, T>(&self, mut f: F) -> Result<Option<T>, Error>
    where
        F: FnMut(&Self) -> Result<T, Error>,
    {
        if self.is_null() {
            return Ok(None);
        }
        if self.is_object() {
            return Ok(Some(f(self)?));
        }
        Err(Error::JsonExpectedObject)
    }

    fn map_opt<F, T>(&self, mut f: F) -> Result<Option<T>, Error>
    where
        F: FnMut(&Self) -> Result<T, Error>,
    {
        if self.is_null() {
            return Ok(None);
        }
        Ok(Some(f(self)?))
    }

    fn to_opt_u32(&self) -> Result<Option<u32>, Error> {
        if self.is_null() {
            return Ok(None);
        }
        let v = self.as_i64().ok_or(Error::JsonExpectedI64)?;
        let vv = u32::try_from(v).map_err(|_| Error::JsonExpectedI64)?;
        Ok(Some(vv))
    }

    fn to_opt_f32(&self) -> Result<Option<f32>, Error> {
        if self.is_null() {
            return Ok(None);
        }
        let v = self.as_f64().ok_or(Error::JsonExpectedF64)?;
        Ok(Some(v as f32))
    }

    fn to_opt_i32(&self) -> Result<Option<i32>, Error> {
        if self.is_null() {
            return Ok(None);
        }
        let v = self.as_i64().ok_or(Error::JsonExpectedI64)?;
        let vv = i32::try_from(v).map_err(|_| Error::JsonExpectedI64)?;
        Ok(Some(vv))
    }

    fn to_opt_string(&self) -> Result<Option<String>, Error> {
        if self.is_null() {
            return Ok(None);
        }
        self.as_str()
            .map(|s| Some(s.to_string()))
            .ok_or(Error::JsonExpectedString)
    }

    fn to_opt_bool(&self) -> Result<Option<bool>, Error> {
        if self.is_null() {
            return Ok(None);
        }
        self.as_bool()
            .map(Some)
            .ok_or(Error::JsonExpectedBool)
    }

    fn to_string_or_err(&self) -> Result<String, Error> {
        self.as_str()
            .map(|s| s.to_string())
            .ok_or(Error::JsonExpectedString)
    }
}
