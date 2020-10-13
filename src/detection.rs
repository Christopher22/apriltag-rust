//! Tag detection types.

use crate::{
    common::*,
    matd::MatdRef,
    pose::{Pose, PoseEstimation},
};

/// Represent a marker detection outcome.
#[repr(transparent)]
pub struct Detection {
    ptr: NonNull<sys::apriltag_detection_t>,
}

impl Detection {
    /// Get the marker ID.
    pub fn id(&self) -> usize {
        unsafe { self.ptr.as_ref().id as usize }
    }

    /// Get the Hamming distance to the target tag.
    pub fn hamming(&self) -> usize {
        unsafe { self.ptr.as_ref().hamming as usize }
    }

    /// Indicate the _goodness_ of the detection.
    pub fn decision_margin(&self) -> f32 {
        unsafe { self.ptr.as_ref().decision_margin }
    }

    /// Get the center coordinates in form of `[x, y]`.
    pub fn center(&self) -> [f64; 2] {
        unsafe { self.ptr.as_ref().c }
    }

    /// Get the corner coordinates in form of `[[x, y]; 4]`.
    pub fn corners(&self) -> [[f64; 2]; 4] {
        unsafe { self.ptr.as_ref().p }
    }

    /// Get the homography matrix.
    pub fn homography(&self) -> MatdRef<'_> {
        unsafe { MatdRef::from_ptr(self.ptr.as_ref().H) }
    }

    /// Estimates the pose of tag with specified number of iterations.
    pub fn estimate_tag_pose_orthogonal_iteration(
        &self,
        params: &TagParams,
        n_iters: usize,
    ) -> Vec<PoseEstimation> {
        let mut info = sys::apriltag_detection_info_t {
            det: self.ptr.as_ptr(),
            tagsize: params.tagsize,
            fx: params.fx,
            fy: params.fy,
            cx: params.cx,
            cy: params.cy,
        };

        let poses: Vec<_> = unsafe {
            let mut pose1: MaybeUninit<sys::apriltag_pose_t> = MaybeUninit::uninit();
            let mut err1: MaybeUninit<f64> = MaybeUninit::uninit();
            let mut pose2: MaybeUninit<sys::apriltag_pose_t> = MaybeUninit::uninit();
            let mut err2: MaybeUninit<f64> = MaybeUninit::uninit();

            sys::estimate_tag_pose_orthogonal_iteration(
                &mut info as *mut _,
                err1.as_mut_ptr(),
                pose1.as_mut_ptr(),
                err2.as_mut_ptr(),
                pose2.as_mut_ptr(),
                n_iters as c_int,
            );

            let pose1 = pose1.assume_init();
            let err1 = err1.assume_init();

            let pose2 = pose2.assume_init();
            let err2 = err2.assume_init();

            let iter1 = if pose1.R == ptr::null_mut() {
                Some(PoseEstimation {
                    pose: Pose(pose1),
                    error: err1,
                })
            } else {
                None
            }
            .into_iter();
            let iter2 = if pose2.R == ptr::null_mut() {
                Some(PoseEstimation {
                    pose: Pose(pose2),
                    error: err2,
                })
            } else {
                None
            }
            .into_iter();

            iter1.chain(iter2).collect()
        };

        // make sure it drops after estimate_tag_pose()
        mem::drop(info);

        poses
    }

    /// Estimates the pose of tag.
    pub fn estimate_tag_pose(&self, params: &TagParams) -> Option<Pose> {
        let mut info = sys::apriltag_detection_info_t {
            det: self.ptr.as_ptr(),
            tagsize: params.tagsize,
            fx: params.fx,
            fy: params.fy,
            cx: params.cx,
            cy: params.cy,
        };

        let pose = unsafe {
            let mut pose: MaybeUninit<sys::apriltag_pose_t> = MaybeUninit::uninit();
            sys::estimate_tag_pose(&mut info as *mut _, pose.as_mut_ptr());
            let pose = pose.assume_init();

            if pose.R == ptr::null_mut() {
                None
            } else {
                Some(Pose(pose))
            }
        };

        // make sure it drops after estimate_tag_pose()
        mem::drop(info);

        pose
    }

    /// Creates an instance from pointer.
    ///
    /// The pointer will be managed by the type. Do not run manual deallocation on the pointer.
    /// Panics if the pointer is null.
    pub unsafe fn from_raw(ptr: *mut sys::apriltag_detection_t) -> Self {
        Self {
            ptr: NonNull::new(ptr).unwrap(),
        }
    }

    /// Returns the underlying pointer.
    pub unsafe fn into_raw(self) -> NonNull<sys::apriltag_detection_t> {
        ManuallyDrop::new(self).ptr
    }
}

impl Debug for Detection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Detection")
            .field("id", &self.id())
            .field("hamming", &self.hamming())
            .field("decision_margin", &self.decision_margin())
            .field("center", &self.center())
            .field("corners", &self.corners())
            .finish()?;
        Ok(())
    }
}

impl Drop for Detection {
    fn drop(&mut self) {
        unsafe {
            sys::apriltag_detection_destroy(self.ptr.as_ptr());
        }
    }
}

/// Stores tag size and camera parameters.
#[derive(Debug, Clone)]
pub struct TagParams {
    pub tagsize: f64,
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64,
}
