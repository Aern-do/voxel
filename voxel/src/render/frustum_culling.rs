use glam::{vec3, Mat4, Vec3, Vec4, Vec4Swizzles};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    normal: Vec3,
    distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn from_vector(vector: Vec4) -> Self {
        let normal = vector.xyz();
        let distance = -vector.w;

        Self { normal, distance }
    }

    pub fn normalize(&self) -> Plane {
        let denom = self.normal.length_recip();
        Plane::new(self.normal * denom, self.distance * denom)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    left_face: Plane,
    right_face: Plane,

    bottom_face: Plane,
    top_face: Plane,

    near_face: Plane,
    far_face: Plane,
}

impl Frustum {
    pub fn from_projection(matrix: Mat4) -> Self {
        let left_face = Plane::from_vector(matrix.row(3) + matrix.row(0)).normalize();
        let right_face = Plane::from_vector(matrix.row(3) - matrix.row(0)).normalize();

        let bottom_face = Plane::from_vector(matrix.row(3) + matrix.row(1)).normalize();
        let top_face = Plane::from_vector(matrix.row(3) - matrix.row(1)).normalize();

        let near_face = Plane::from_vector(matrix.row(3) + matrix.row(2)).normalize();
        let far_face = Plane::from_vector(matrix.row(3) - matrix.row(2)).normalize();

        Self {
            left_face,
            right_face,
            bottom_face,
            top_face,
            near_face,
            far_face,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Plane> {
        [
            &self.left_face,
            &self.right_face,
            &self.bottom_face,
            &self.top_face,
            &self.near_face,
            &self.far_face,
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    fn is_point_on_plane(plane: &Plane, point: Vec3) -> bool {
        let distance = point.dot(plane.normal);
        distance >= plane.distance
    }

    pub fn is_on_plane(self, plane: &Plane) -> bool {
        let corners = [
            self.min,
            vec3(self.max.x, self.min.y, self.min.z),
            vec3(self.min.x, self.max.y, self.min.z),
            vec3(self.max.x, self.max.y, self.min.z),
            vec3(self.min.x, self.min.y, self.max.z),
            vec3(self.max.x, self.min.y, self.max.z),
            vec3(self.min.x, self.max.y, self.max.z),
            self.max,
        ];

        let first = AABB::is_point_on_plane(plane, corners[0]);
        for point in corners[1..].iter() {
            if AABB::is_point_on_plane(plane, *point) != first {
                return true;
            }
        }
        first
    }

    pub fn is_on_frustum(&self, frustum: &Frustum) -> bool {
        frustum.iter().all(|plane| self.is_on_plane(plane))
    }
}
