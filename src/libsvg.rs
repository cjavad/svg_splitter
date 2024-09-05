use ori::prelude::Point;
use std::ffi;

#[repr(C)]
struct cImportSvgOptions {
    fn_: ffi::c_double,
    fs: ffi::c_double,
    fa: ffi::c_double,
    dpi: ffi::c_double,
    center: bool,
}

#[repr(C)]
struct cSplitPolygonOptions {
    point_in_point_check: bool,
}

#[repr(C)]
#[derive(Debug)]
struct cVector2D {
    pub x: ffi::c_double,
    pub y: ffi::c_double,
}

#[repr(C)]
#[derive(Debug)]
struct cOutline {
    pub len: usize,
    pub positive: bool,
    pub vertices: *mut cVector2D,
}

#[derive(Debug)]
pub struct Outline {
    pub bbox: (Point, Point),
    pub vertices: Vec<Point>,
    pub positive: bool,
}

#[link(name = "OpenScadLibSvgCFFI")]
extern "C" {
    pub fn svg_to_polygon_c(
        svg: *const ffi::c_char,
        size: usize,
        options: *mut cImportSvgOptions,
        polygon: *mut *mut ffi::c_void,
    ) -> ffi::c_int;

    pub fn get_polygon_outlines(
        polygon: *mut ffi::c_void,
        outlines: *mut *mut cOutline,
        size: *mut usize,
    ) -> ffi::c_int;

    pub fn polygon_to_svg_c(
        polygon: *const *mut ffi::c_void,
        svg: *mut *mut ffi::c_char,
        size: *mut usize,
    ) -> ffi::c_int;

    pub fn split_polygon_c(
        polygon: *mut ffi::c_void,
        options: *mut cSplitPolygonOptions,
        polygons: *mut *mut *mut ffi::c_void,
        size: *mut usize,
    ) -> ffi::c_int;
    pub fn free_polygon(polygon: *mut ffi::c_void) -> ffi::c_int;
    pub fn free_polygons(polygons: *mut *mut ffi::c_void, size: usize) -> ffi::c_int;

    pub fn free_outline(outline: *mut cOutline) -> ffi::c_int;
    pub fn free_outlines(outlines: *mut cOutline, size: usize) -> ffi::c_int;
}

unsafe fn get_outlines(c_polygon: *mut ffi::c_void) -> Result<Vec<Outline>, String> {
    let ret;

    let mut c_outlines = std::ptr::null_mut();
    let mut size = 0;

    ret = get_polygon_outlines(c_polygon, &mut c_outlines, &mut size);

    if ret != 0 {
        return Err("Failed to get outlines".to_string());
    }

    let mut outlines = Vec::new();

    for i in 0..size {
        let c_outline = &*c_outlines.offset(i as isize);

        let mut vertices = Vec::new();

        for j in 0..c_outline.len {
            let c_vertex = &*c_outline.vertices.offset(j as isize);
            vertices.push(Point::new(c_vertex.x as f32, c_vertex.y as f32));
        }

        outlines.push(Outline::new(vertices, c_outline.positive));
    }

    Ok(outlines)
}

pub fn svg_process(svg_data: &str, pip: bool) -> Result<Vec<Vec<Outline>>, String> {
    let mut ret;

    let mut options = cImportSvgOptions {
        fn_: 1.0,
        fs: 1.0,
        fa: 1.0,
        dpi: 72.0,
        center: true,
    };

    let mut polygon = std::ptr::null_mut();

    // Convert SVG to polygon
    unsafe {
        ret = svg_to_polygon_c(
            svg_data.as_ptr() as *const i8,
            svg_data.len(),
            &mut options,
            &mut polygon,
        );
    }

    if ret != 0 {
        return Err("Failed to convert SVG to polygon".to_string());
    }
    
    // Split polygon
    let mut split_options = cSplitPolygonOptions {
        point_in_point_check: pip,
    };

    let mut outlines = Vec::new();

    // Store the original outline of the polygon.
    unsafe {
        outlines.push(get_outlines(polygon)?);
    }

    let mut c_list_of_polygons = std::ptr::null_mut();
    let mut size = 0;

    // Maybe this drops it.
    unsafe {
        ret = split_polygon_c(
            polygon,
            &mut split_options,
            &mut c_list_of_polygons,
            &mut size,
        );
    }

    if ret != 0 {
        return Err("Failed to split polygon".to_string());
    }

    // Convert the list of polygons to outlines
    for i in 0..size {
        unsafe {
            let c_polygon = c_list_of_polygons.offset(i as isize);
            outlines.push(get_outlines(*c_polygon)?);
        }
    }

    // Free the list of polygons
    unsafe {
        // ret = free_polygons(c_list_of_polygons, size);
    }

    if ret != 0 {
        return Err("Failed to free polygons".to_string());
    }

    Ok(outlines)
}

impl Outline {
    pub fn new(vertices: Vec<Point>, positive: bool) -> Self {
        let bbox = Self::bbox(&vertices);
        Self { vertices, positive, bbox }
    }

    pub fn bbox(vertices: &[Point]) -> (Point, Point) {
        let mut min = Point::new(f32::MAX, f32::MAX);
        let mut max = Point::new(f32::MIN, f32::MIN);

        for vertex in vertices {
            if vertex.x < min.x {
                min.x = vertex.x;
            }

            if vertex.y < min.y {
                min.y = vertex.y;
            }

            if vertex.x > max.x {
                max.x = vertex.x;
            }

            if vertex.y > max.y {
                max.y = vertex.y;
            }
        }

        (min, max)
    }
}
