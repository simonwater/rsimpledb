use crate::parse::QueryData;

pub struct CreateViewData {
    viewname: String,
    qrydata: QueryData,
}

impl CreateViewData {
    pub fn new(viewname: String, qrydata: QueryData) -> Self {
        CreateViewData { viewname, qrydata }
    }

    pub fn view_name(&self) -> &str {
        &self.viewname
    }

    pub fn view_def(&self) -> String {
        self.qrydata.to_string()
    }

    pub fn query_data(&self) -> &QueryData {
        &self.qrydata
    }
}

