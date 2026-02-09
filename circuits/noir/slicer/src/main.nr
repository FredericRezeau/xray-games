/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau <hello@kyungj.in>
*/

use poseidon::poseidon2::Poseidon2;

global MAX_POLYGONS: u32 = 4;
global MAX_VERTICES_PER_POLYGON: u32 = 14;
global MAX_PARTITIONS: u32 = 20;
global MAX_VERTICES_PER_PARTITION: u32 = 14;
global MAX_OBJECTS: u32 = 15;
global MAX_SEGMENTS: u32 = 3;
global TOLERANCE: i64 = 10000; // Tolerance for colinearity checks.

struct Point {
    x: i64,
    y: i64,
}

struct Segment {
    x1: i64,
    y1: i64,
    x2: i64,
    y2: i64,
}

struct Polygon {
    vertices: [Point; MAX_VERTICES_PER_POLYGON],
    vertex_count: u32,
}

struct Partition {
    vertices: [Point; MAX_VERTICES_PER_PARTITION],
    vertex_count: u32,
}

struct EdgeHint {
    source_type: u8,
    source_index: u32,
    edge_index: u32,
}

struct PieceEdgeHints {
    hints: [EdgeHint; MAX_VERTICES_PER_PARTITION],
}

fn to_field(value: i64) -> Field {
    if value >= 0 {
        (value as u64) as Field
    } else {
        let pos = (-value) as u64;
        0 - (pos as Field)
    }
}

fn hash2(a: Field, b: Field) -> Field {
    Poseidon2::hash([a, b], 2)
}

fn hash_level(polygons: [Polygon; MAX_POLYGONS], polygon_count: u32,
    objects: [Point; MAX_OBJECTS], object_count: u32, segment_count: u32
) -> Field {
    let mut hash: Field = 0;
    for p in 0..MAX_POLYGONS {
        if p < polygon_count {
            let polygon = polygons[p];
            for v in 0..MAX_VERTICES_PER_POLYGON {
                if v < polygon.vertex_count {
                    hash = hash2(hash, to_field(polygon.vertices[v].x));
                    hash = hash2(hash, to_field(polygon.vertices[v].y));
                }
            }
            hash = hash2(hash, 0xFFFFFFFF);
        }
    }

    for o in 0..MAX_OBJECTS {
        if o < object_count {
            hash = hash2(hash, to_field(objects[o].x));
            hash = hash2(hash, to_field(objects[o].y));
        }
    }

    hash = hash2(hash, segment_count as Field);
    hash
}

fn cross(point1: Point, point2: Point, point3: Point) -> i64 {
    (point2.x - point1.x) * (point3.y - point1.y)
    - (point2.y - point1.y) * (point3.x - point1.x)
}

fn abs(value: i64) -> i64 {
    if value < 0 {
        -value
    } else {
        value
    }
}

fn point_on_line(point: Point, start: Point, end: Point) -> bool {
    abs(cross(start, end, point)) < TOLERANCE
}

fn edge_on_line(edge_start: Point, edge_end: Point,
    line_start: Point, line_end: Point) -> bool {
    point_on_line(edge_start, line_start, line_end)
    & point_on_line(edge_end, line_start, line_end)
}

fn point_in_polygon(point: Point, vertices: [Point; MAX_VERTICES_PER_PARTITION], count: u32) -> bool {
    let mut winding: i64 = 0;
    for i in 0..MAX_VERTICES_PER_PARTITION {
        if i < count {
            let point1 = vertices[i];
            let point2 = vertices[(i + 1) % count];
            let result = cross(point1, point2, point);
            if (point1.y <= point.y) & (point2.y > point.y) & (result > 0) {
                winding += 1;
            }
            if (point1.y > point.y) & (point2.y <= point.y) & (result < 0) {
                winding -= 1;
            }
        }
    }

    winding != 0
}

fn verify_edge_with_hint(edge_start: Point, edge_end: Point, hint: EdgeHint,
    polygons: [Polygon; MAX_POLYGONS], segments: [Segment; MAX_SEGMENTS]) -> bool {
    if hint.source_type == 0 {
        let polygon = polygons[hint.source_index];
        let index = hint.edge_index;
        edge_on_line(edge_start, edge_end, polygon.vertices[index],
            polygon.vertices[(index + 1) % polygon.vertex_count])
    } else {
        let segment = segments[hint.source_index];
        let start = Point { x: segment.x1, y: segment.y1 };
        let end = Point { x: segment.x2, y: segment.y2 };
        edge_on_line(edge_start, edge_end, start, end)
    }
}

fn verify_all_edges(partitions: [Partition; MAX_PARTITIONS], partition_count: u32,
    edge_hints: [PieceEdgeHints; MAX_PARTITIONS], polygons: [Polygon; MAX_POLYGONS],
    segments: [Segment; MAX_SEGMENTS]) -> bool {
    let mut valid = true;
    for p in 0..MAX_PARTITIONS {
        if p < partition_count {
            let partition = partitions[p];
            for i in 0..MAX_VERTICES_PER_PARTITION {
                if i < partition.vertex_count {
                    let j = (i + 1) % partition.vertex_count;
                    if !verify_edge_with_hint(partition.vertices[i], partition.vertices[j],
                        edge_hints[p].hints[i], polygons, segments) {
                        valid = false;
                    }
                }
            }
        }
    }

    valid
}

fn verify_object_placement(objects: [Point; MAX_OBJECTS], object_count: u32,
    partitions: [Partition; MAX_PARTITIONS], partition_count: u32,
    object_hints: [u32; MAX_OBJECTS]) -> bool {
    let mut valid = true;
    let mut claimed: [bool; MAX_PARTITIONS] = [false; MAX_PARTITIONS];
    for o in 0..MAX_OBJECTS {
        if o < object_count {
            let object = objects[o];
            let index = object_hints[o];
            if index >= partition_count {
                valid = false;
            } else {
                let partition = partitions[index];
                if !point_in_polygon(object, partition.vertices, partition.vertex_count) {
                    valid = false;
                }
                if claimed[index] {
                    valid = false;
                }
                claimed[index] = true;
            }
        }
    }

    valid
}

fn main(
    // Public inputs.
    level_hash: pub Field,
    polygon_count: pub u32,
    object_count: pub u32,
    partition_count: pub u32,

    // Private inputs.
    polygons: [Polygon; MAX_POLYGONS],
    objects: [Point; MAX_OBJECTS],
    segment_count: u32,
    segments: [Segment; MAX_SEGMENTS],
    partitions: [Partition; MAX_PARTITIONS],
    edge_hints: [PieceEdgeHints; MAX_PARTITIONS],
    object_hints: [u32; MAX_OBJECTS]
) {
    assert(hash_level(polygons, polygon_count, objects, object_count, segment_count) == level_hash);
    assert(partition_count >= object_count);
    assert(verify_all_edges(partitions, partition_count, edge_hints, polygons, segments));
    assert(verify_object_placement(objects, object_count, partitions, partition_count, object_hints));
}

#[test]
fn test_cross_product() {
    let point1 = Point { x: 0, y: 0 };
    let point2 = Point { x: 10, y: 0 };
    let point3 = Point { x: 5, y: 5 };
    assert(cross(point1, point2, point3) > 0);
}

#[test]
fn test_point_on_line() {
    let start = Point { x: 0, y: 0 };
    let end = Point { x: 100, y: 100 };
    let test = Point { x: 50, y: 50 };
    assert(point_on_line(test, start, end));
}

#[test]
fn test_point_in_polygon() {
    let mut vertices: [Point; MAX_VERTICES_PER_PARTITION] = [Point { x: 0, y: 0 }; MAX_VERTICES_PER_PARTITION];
    vertices[0] = Point { x: 0, y: 0 };
    vertices[1] = Point { x: 100, y: 0 };
    vertices[2] = Point { x: 100, y: 100 };
    vertices[3] = Point { x: 0, y: 100 };
    let test = Point { x: 50, y: 50 };
    assert(point_in_polygon(test, vertices, 4));
}

#[test]
fn test_hash_deterministic() {
    assert(hash2(1, 2) == hash2(1, 2));
}
