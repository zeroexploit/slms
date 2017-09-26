/// # NameValuePair
///
/// A simple and universal Name - Value Pair
///
/// Will be used most likely to interact with XML Data
/// and the XMLParser in Order to provide a XML Tags
/// Attributes.
#[derive(Clone)]
pub struct NameValuePair {
    pub name: String,
    pub value: String,
}

impl NameValuePair {
    /// Creates a new Name Value Pair with the given Name and Value
    ///
    /// # Arguments
    ///
    /// * `name` - A String containing the Key Name
    /// * `value` - String containting the Value to the Key
    ///
    /// # Example
    ///
    /// ```
    /// use tools::NameValuePair
    /// let pair = NameValuePair::new("name", "value");
    /// ```
    pub fn new(name: &str, value: &str) -> NameValuePair {
        NameValuePair {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}
