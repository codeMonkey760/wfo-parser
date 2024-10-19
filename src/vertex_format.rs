#[derive(PartialEq, Eq, Debug)]
pub(crate) enum VertexFormat {
    Unknown,
    VertexP,
    VertexPN,
    VertexPT,
    VertexPNT,
}

impl VertexFormat {
    pub(crate) fn from_indices(indices: &(u64, u64, u64)) -> Self {
        match indices {
            (0, n, tc) => { panic!("Vertex format must have position index"); },
            (p, 0, 0) => VertexFormat::VertexP,
            (p, n, 0) => VertexFormat::VertexPN,
            (p, 0, tc) => VertexFormat::VertexPT,
            (ps, n, tc) => VertexFormat::VertexPNT,
        }
    }
}
