use crate::{errors::Error, indexes::Index};
use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct MatchRange {
    start: usize,
    length: usize
}

/// A single result.  
/// Contains the complete object, optionally the formatted object, and optionaly an object that contains information about the matches.
#[derive(Deserialize, Debug)]
pub struct SearchResult<T> {
    /// The full result.
    #[serde(flatten)]
    pub result: T,
    /// The formatted result.
    #[serde(rename = "_formatted")]
    pub formatted_result: Option<T>,
    /// The object that contains information about the matches.
    #[serde(rename = "_matchesInfo")]
    pub matches_info: Option<HashMap<String, Vec<MatchRange>>>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// A struct containing search results and other information about the search.
pub struct SearchResults<T> {
    /// results of the query
    pub hits: Vec<SearchResult<T>>,
    /// number of documents skipped
    pub offset: usize,
    /// number of documents to take
    pub limit: usize,
    /// total number of matches
    pub nb_hits: usize,
    /// whether nbHits is exhaustive
    pub exhaustive_nb_hits: bool,
    /// Distribution of the given facets.
    pub facets_distribution: Option<HashMap<String, HashMap<String, usize>>>,
    /// Whether facet_distribution is exhaustive
    pub exhaustive_facets_count: Option<bool>,
    /// processing time of the query
    pub processing_time_ms: usize,
    /// query originating the response
    pub query: String,
}

fn serialize_with_wildcard<S, T>(data: &Option<Option<T>>, s: S) -> Result<S::Ok, S::Error> where S: Serializer, T: Serialize {
    match data {
        Some(None) => s.serialize_str("*"),
        Some(data) => data.serialize(s),
        None => s.serialize_none(),
    }
}

type AttributeToCrop<'a> = (&'a str, Option<usize>);

/// A struct representing a query.
/// You can add search parameters using the builder syntax.
/// See [here](https://docs.meilisearch.com/guides/advanced_guides/search_parameters.html#query-q) for the list and description of all parameters.
///
/// # Example
///
/// ```
/// # use meilisearch_sdk::search::Query;
/// let query = Query::new("space")
///     .with_offset(42)
///     .with_limit(21)
///     .build();
/// ```
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")] 
pub struct Query<'a> {
    /// The query parameter is the only mandatory parameter.
    /// This is the string used by the search engine to find relevant documents.
    #[serde(rename = "q")]
    pub query: &'a str,
    /// A number of documents to skip. If the value of the parameter offset is n, n first documents to skip. This is helpful for pagination.
    ///
    /// Example: If you want to skip the first document, set offset to 1.
    /// Default: 0
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub offset: Option<usize>,
    /// Set a limit to the number of documents returned by search queries. If the value of the parameter limit is n, there will be n documents in the search query response. This is helpful for pagination.
    ///
    /// Example: If you want to get only two documents, set limit to 2.
    /// Default: 20
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub limit: Option<usize>,
    /// Specify a filter to be used with the query. See the [dedicated guide](https://docs.meilisearch.com/guides/advanced_guides/filtering.html).
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub filters: Option<&'a str>,
    /// Facet names and values to filter on. See [this page](https://docs.meilisearch.com/guides/advanced_guides/search_parameters.html#facet-filters).
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub facet_filters: Option<&'a [&'a [&'a str]]>,
    /// Facets for which to retrieve the matching count. The value `Some(None)` is the wildcard.
    #[serde(skip_serializing_if = "Option::is_none")] 
    #[serde(serialize_with = "serialize_with_wildcard")]
    pub facets_distribution: Option<Option<&'a [&'a str]>>,
    /// Attributes to **display** in the returned documents.
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub attributes_to_retrieve: Option<&'a [&'a str]>,
    /// Attributes to crop. The value `Some(None)` is the wildcard. Attributes are composed by the attribute name and an optional `usize` that overwrites the `crop_length` parameter.
    #[serde(skip_serializing_if = "Option::is_none")] 
    #[serde(serialize_with = "serialize_with_wildcard")]
    pub attributes_to_crop: Option<Option<&'a [AttributeToCrop<'a>]>>,
    /// Number of characters to keep on each side of the start of the matching word. See [attributes_to_crop](#structfield.attributes_to_crop).
    ///
    /// Default: 200
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub crop_length: Option<usize>,
    /// Attributes whose values will contain **highlighted matching query words**. The value `Some(None)` is the wildcard.
    #[serde(skip_serializing_if = "Option::is_none")] 
    #[serde(serialize_with = "serialize_with_wildcard")]
    pub attributes_to_highlight: Option<Option<&'a [&'a str]>>,
    /// Defines whether an object that contains information about the matches should be returned or not
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub matches: Option<bool>
}

#[allow(missing_docs)]
impl<'a> Query<'a> {
    pub fn new(query: &'a str) -> Query<'a> {
        Query {
            query,
            offset: None,
            limit: None,
            filters: None,
            facet_filters: None,
            facets_distribution: None,
            attributes_to_retrieve: None,
            attributes_to_crop: None,
            attributes_to_highlight: None,
            crop_length: None,
            matches: None,
        }
    }
    pub fn with_offset<'b>(&'b mut self, offset: usize) -> &'b mut Query<'a> {
        self.offset = Some(offset);
        self
    }
    pub fn with_limit<'b>(&'b mut self, limit: usize) -> &'b mut Query<'a> {
        self.limit = Some(limit);
        self
    }
    pub fn with_filters<'b>(&'b mut self, filters: &'a str) -> &'b mut Query<'a> {
        self.filters = Some(filters);
        self
    }
    pub fn with_facet_filters<'b>(&'b mut self, facet_filters: &'a[&'a[&'a str]]) -> &'b mut Query<'a> {
        self.facet_filters = Some(facet_filters);
        self
    }
    pub fn with_facets_distribution<'b>(&'b mut self, facets_distribution: Option<&'a[&'a str]>) -> &'b mut Query<'a> {
        self.facets_distribution = Some(facets_distribution);
        self
    }
    pub fn with_attributes_to_retrieve<'b>(&'b mut self, attributes_to_retrieve: &'a [&'a str]) -> &'b mut Query<'a> {
        self.attributes_to_retrieve = Some(attributes_to_retrieve);
        self
    }
    pub fn with_attributes_to_crop<'b>(&'b mut self, attributes_to_crop: Option<&'a [(&'a str, Option<usize>)]>) -> &'b mut Query<'a> {
        self.attributes_to_crop = Some(attributes_to_crop);
        self
    }
    pub fn with_attributes_to_highlight<'b>(&'b mut self, attributes_to_highlight: Option<&'a [&'a str]>) -> &'b mut Query<'a> {
        self.attributes_to_highlight = Some(attributes_to_highlight);
        self
    }
    pub fn with_crop_length<'b>(&'b mut self, crop_length: usize) -> &'b mut Query<'a> {
        self.crop_length = Some(crop_length);
        self
    }
    pub fn with_matches<'b>(&'b mut self, matches: bool) -> &'b mut Query<'a> {
        self.matches = Some(matches);
        self
    }
    pub fn build(&mut self) -> Query<'a> {
        self.clone()
    }
}

impl<'a> Query<'a> {
    /// Alias for [the Index method](../indexes/struct.Index.html#method.search).
    pub async fn execute<T: 'static + DeserializeOwned>(
        &'a self,
        index: &Index<'a>,
    ) -> Result<SearchResults<T>, Error> {
        index.search::<T>(&self).await
    }
}
