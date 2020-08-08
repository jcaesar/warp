//! Query Filters

use futures::future;
use serde::de::DeserializeOwned;
use serde_urlencoded;

use crate::filter::{filter_fn_one, Filter, One};
use crate::reject::{self, Rejection};

/// Creates a `Filter` that decodes query parameters to the type `T`.
///
/// If cannot decode into a `T`, the request is rejected with a `400 Bad Request`.
///
/// Currently the decoder strictly follows the URL spec, hence some enums and structs
/// (e.g. structs containing `Vec`s) cannot be decoded into.
/// All structs containing only scalar types and strings, and `Vec<(String, String)>`
/// can be decoded into.
pub fn query<T: DeserializeOwned + Send + 'static>(
) -> impl Filter<Extract = One<T>, Error = Rejection> + Copy {
    filter_fn_one(|route| {
        let query_string = route.query().unwrap_or_else(|| {
            tracing::debug!("route was called without a query string, defaulting to empty");
            ""
        });

        let query_encoded = serde_urlencoded::from_str(query_string).map_err(|e| {
            tracing::debug!("failed to decode query string '{}': {:?}", query_string, e);
            reject::invalid_query()
        });
        future::ready(query_encoded)
    })
}

/// Creates a `Filter` that returns the raw query string as type String.
///
/// Can be used to implement a filter with a custom deserializer, for example `serde_qs`.
///
/// ```
/// # fn main() {
/// # use serde::de::DeserializeOwned;
/// # use serde_derive::Deserialize;
/// # mod serde_qs { pub fn from_str<T>(_: &str) -> Result<T, ()> { unreachable!() } }
/// # use warp::Filter;
/// fn query_qs<T: DeserializeOwned>()
///     -> impl Filter<Extract = (T,), Error = warp::Rejection> + Copy
/// {
///     warp::query::raw().and_then(|q: String| async move {
///         serde_qs::from_str::<T>(&q).map_err(|e| warp::reject::reject())
///     })
/// }
///
/// #[derive(Deserialize, Debug)]
/// struct Pars { q: Vec<u64> }
///
/// let product = warp::path("product").and(query_qs::<Pars>())
///     .map(|product: Pars| format!("{}", product.q.into_iter().product::<u64>()));
///
/// # }
/// ```

pub fn raw() -> impl Filter<Extract = One<String>, Error = Rejection> + Copy {
    filter_fn_one(|route| {
        let route = route
            .query()
            .map(|q| q.to_owned())
            .map(Ok)
            .unwrap_or_else(|| Err(reject::invalid_query()));
        future::ready(route)
    })
}
