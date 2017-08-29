/// # NameValuePair
///
/// A simple and universal Name - Value Pair
/// 
/// Will be used most likely to interact with XML Data
/// and the XMLParser in Order to provide a XML Tags
/// Attributes.
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

    /// Set both, the Name and Value for this Pair
    ///
    /// # Arguments
    ///
    /// * `name` - A String containing the Keys Name
    /// * `value` - A String containing the Value
    ///
    /// # Example
    ///
    /// ```
    /// let pair = NameValuePair::new("old_name", "old_value");
    /// pair.set("new_name", "new_value");
    /// ```
    pub fn set(&mut self, name: &str, value: &str) {
        self.name = name.to_string();
        self.value = value.to_string();
    }

    /// Get both, the Name and Value of this Pair as Tupel
    ///
    /// # Example
    ///
    /// ```
    /// let pair = NameValuePair::new("name", "value");
    /// let (name, value) = pair.get();
    /// ```
    pub fn get(&self) -> (&str, &str) {
        (&self.name, &self.value)
    }
    
    /// Copy the Name Value Pair and create a new one.
    /// Both Pairs can be used independently.
    ///
    /// # Example
    ///
    /// ```
    /// let other_pair: NameValuePair = NameValuePair::new();
    /// let pair: NameValuePair = other_pair.copy();
    /// ```
    pub fn copy(&self) -> NameValuePair {
        NameValuePair {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}
