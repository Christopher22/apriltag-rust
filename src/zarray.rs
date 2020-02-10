use apriltag_sys::zarray_t;
use std::{
    convert::AsRef,
    ffi::c_void,
    marker::PhantomData,
    ops::{Index, IndexMut},
    os::raw::c_char,
    ptr::NonNull,
};

#[derive(Debug, Clone)]
pub struct ZarrayIter<'a, T> {
    zarray: &'a Zarray<T>,
    len: usize,
    index: usize,
}

impl<'a, T> Iterator for ZarrayIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let index = self.index;
            self.index += 1;
            Some(&self.zarray[index])
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Zarray<T> {
    ptr: NonNull<zarray_t>,
    phantom: PhantomData<T>,
}

impl<T> Zarray<T> {
    pub fn _new() -> Self {
        let ptr = unsafe {
            let ptr = libc::calloc(1, std::mem::size_of::<zarray_t>()) as *mut zarray_t;
            *ptr.as_mut().unwrap() = zarray_t {
                el_sz: std::mem::size_of::<T>() as u64,
                size: 0,
                alloc: 0,
                data: std::ptr::null_mut(),
            };
            ptr
        };
        Self {
            ptr: NonNull::new(ptr).unwrap(),
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        unsafe { self.ptr.as_ref().size as usize }
    }

    pub fn iter(&self) -> ZarrayIter<T> {
        ZarrayIter {
            zarray: self,
            len: self.len(),
            index: 0,
        }
    }

    pub unsafe fn from_ptr(ptr: NonNull<zarray_t>) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<T> AsRef<[T]> for Zarray<T> {
    fn as_ref(&self) -> &[T] {
        unsafe {
            let as_ref = self.ptr.as_ref();
            std::slice::from_raw_parts(as_ref.data as *const T, as_ref.size as usize)
        }
    }
}

impl<T> AsMut<[T]> for Zarray<T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe {
            let as_mut = self.ptr.as_mut();
            std::slice::from_raw_parts_mut(as_mut.data as *mut T, as_mut.size as usize)
        }
    }
}

impl<T> Index<usize> for Zarray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_ref()[index]
    }
}

impl<T> IndexMut<usize> for Zarray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut()[index]
    }
}

impl<T> Clone for Zarray<T> {
    fn clone(&self) -> Self {
        let ptr = unsafe {
            let from_ptr = self.ptr.as_ptr();
            let to_ptr = libc::calloc(1, std::mem::size_of::<zarray_t>()) as *mut zarray_t;

            let zarray_t {
                el_sz,
                size,
                alloc,
                data: from_data,
            } = *from_ptr;
            assert!(size <= alloc);
            assert_eq!(el_sz as usize, std::mem::size_of::<T>());

            let to_data = libc::malloc(alloc as usize * el_sz as usize);
            libc::memcpy(
                to_data,
                from_data as *mut c_void,
                size as usize * el_sz as usize,
            );

            *to_ptr.as_mut().unwrap() = zarray_t {
                el_sz,
                size,
                alloc,
                data: to_data as *mut c_char,
            };
            to_ptr
        };

        Self {
            ptr: NonNull::new(ptr).unwrap(),
            phantom: PhantomData,
        }
    }
}

impl<T> Drop for Zarray<T> {
    fn drop(&mut self) {
        unsafe {
            let data_ptr = self.ptr.as_mut().data;
            if !data_ptr.is_null() {
                libc::free(data_ptr as *mut c_void);
            }
            libc::free(self.ptr.as_ptr() as *mut c_void);
        }
    }
}
