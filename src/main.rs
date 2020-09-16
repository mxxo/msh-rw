use msh_rw::*;

fn main() {
    let mut msh = Msh::new();
    msh.nodes = vec![
        Node { tag: 1, x: 0.0, y: 0.0, z: 0.0, },
        Node { tag: 2, x: 1.0, y: 0.0, z: 0.0, },
    ];
    msh.elts = vec![MeshElt {
            tag: 1,
            ty: MeshShape::Line,
            nodes: vec![1, 2],
            physical_group: None,
            geometry: None,
        }
    ];

    println!("{:#?}", &msh);
}
