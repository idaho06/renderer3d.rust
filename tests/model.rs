//! Integration tests for [`render3d::model::Model`] OBJ parsing.
//!
//! These tests exercise `Model::from_obj_and_png` against the small fixture
//! files in `tests/fixtures/`.  For geometry-only tests `assets/cube.png` is
//! used as a stand-in texture — the texture contents are irrelevant to what
//! the tests verify.

use render3d::model::Model;

const DUMMY_PNG: &str = "assets/cube.png";

// ── triangle.obj ────────────────────────────────────────────────────────────

#[test]
fn triangle_vertex_uv_face_counts() {
    let model = Model::from_obj_and_png("tests/fixtures/triangle.obj", DUMMY_PNG);
    assert_eq!(model.vertices.len(), 3, "expected 3 vertices");
    assert_eq!(model.uvs.len(), 3, "expected 3 UVs");
    assert_eq!(model.normals.len(), 1, "expected 1 normal");
    assert_eq!(model.faces.len(), 1, "expected 1 face");
}

#[test]
fn triangle_uv_v_is_flipped() {
    // The OBJ loader stores UVs as (u, 1.0 - v) to convert from OBJ convention
    // (origin bottom-left) to texture convention (origin top-left).
    let model = Model::from_obj_and_png("tests/fixtures/triangle.obj", DUMMY_PNG);
    // triangle.obj has: vt 0.0 0.0 → stored as (0.0, 1.0)
    assert!(
        (model.uvs[0].y - 1.0).abs() < 1e-5,
        "vt v=0.0 should be stored as 1.0 after flip"
    );
    // triangle.obj has: vt 0.5 1.0 → stored as (0.5, 0.0)
    assert!(
        (model.uvs[2].y - 0.0).abs() < 1e-5,
        "vt v=1.0 should be stored as 0.0 after flip"
    );
}

// ── quad.obj ─────────────────────────────────────────────────────────────────

#[test]
fn quad_triangulates_to_two_faces() {
    let model = Model::from_obj_and_png("tests/fixtures/quad.obj", DUMMY_PNG);
    assert_eq!(model.vertices.len(), 4, "expected 4 vertices");
    assert_eq!(model.uvs.len(), 4, "expected 4 UVs");
    assert_eq!(model.faces.len(), 2, "one quad should triangulate to 2 faces");
}

#[test]
fn quad_face_vertex_indices() {
    let model = Model::from_obj_and_png("tests/fixtures/quad.obj", DUMMY_PNG);
    // Triangulation rule: [v0,v1,v2] and [v0,v2,v3] (0-based, OBJ indices are 1-based).
    assert_eq!(model.faces[0].vertices, [0, 1, 2]);
    assert_eq!(model.faces[1].vertices, [0, 2, 3]);
}

// ── no_normals.obj ───────────────────────────────────────────────────────────

#[test]
#[should_panic]
fn no_normals_panics() {
    // The parser requires v/vt/vn face format.  A file using v/vt (no normals)
    // causes an unwrap on None when reading the missing normal index token.
    // This test documents the assumption: normals are required in all OBJ files.
    let _ = Model::from_obj_and_png("tests/fixtures/no_normals.obj", DUMMY_PNG);
}
