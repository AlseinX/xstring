use serde::{Deserialize, Serialize};

use crate::{Allocator, Str, XString};

impl<T: Str + ?Sized + Serialize, A: Allocator> Serialize for XString<T, A> {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        T::serialize(self, serializer)
    }
}

impl<'de, T: Str + ?Sized, A: Allocator + Default> Deserialize<'de> for XString<T, A>
where
    &'de T: Deserialize<'de> + 'de,
{
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new_in(<&T>::deserialize(deserializer)?, A::default()))
    }
}
