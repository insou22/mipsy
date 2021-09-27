use std::mem;

pub(super) enum UnsafeCow<T>
where
    T: ?Sized
{
    Owned(Box<T>),
    Borrowed(*const T),
}

impl<T> UnsafeCow<T>
where
    T: ?Sized,
{
    pub(super) fn new(value: T) -> Self
    where
        T: Sized
    {
        UnsafeCow::Owned(Box::new(value))
    }

    pub(super) fn new_boxed(value: Box<T>) -> Self {
        UnsafeCow::Owned(value)
    }

    pub(super) fn to_borrowed(&self) -> Self {
        match self {
            Self::Owned(value) => UnsafeCow::Borrowed(value.as_ref() as _),
            Self::Borrowed(value) => UnsafeCow::Borrowed(*value),
        }
    }

    /// Returns a reference to the value.
    /// 
    /// # Safety
    ///
    /// It is up to the caller to guarantee that any
    /// references to the value are valid.
    /// 
    /// If the Cow is owned, this function is perfectly safe.
    ///
    /// However, if the Cow is borrowed, this function is unsafe.
    pub(super) unsafe fn unsafe_borrow<'s>(&'s self) -> &'s T {
        match self {
            Self::Owned(value) => value.as_ref(),

            // SAFETY: given by invariant of `UnsafeCow`
            Self::Borrowed(value) => { 
                let value: &'s T = unsafe { value.as_ref() }.unwrap();

                value
            },
        }
    }

    /// Returns a mutable reference to the value.
    /// 
    /// # Safety
    ///
    /// If the Cow is owned, this function is perfectly safe.
    ///
    /// However, if the Cow is borrowed, this function is unsafe.
    ///
    /// It is up to the caller to guarantee that any
    /// borrowed references to the value remain valid.
    pub(super) unsafe fn unsafe_borrow_mut<'s>(&'s mut self) -> &'s mut T
    where
        T: Clone,
    {
        match self {
            Self::Owned(value) => value.as_mut(),
            Self::Borrowed(value) => {
                // SAFETY: caller must ensure that value is a valid reference
                let value: &'s T = unsafe { value.as_ref() }.unwrap();

                mem::replace(self, Self::new(value.clone()));

                match self {
                    Self::Owned(value) => value.as_mut(),
                    Self::Borrowed(_)  => unreachable!(),
                }
            }
        }
    }
}

impl<T> UnsafeCow<[T]>
{
    /// Returns a mutable reference to the slice.
    /// 
    /// # Safety
    ///
    /// If the Cow is owned, this function is perfectly safe.
    ///
    /// However, if the Cow is borrowed, this function is unsafe.
    ///
    /// It is up to the caller to guarantee that any
    /// borrowed references to the slice remain valid.
    pub(super) unsafe fn unsafe_borrow_mut_slice<'s>(&'s mut self) -> &'s mut [T]
    where
        T: Clone,
    {
        match self {
            Self::Owned(value) => value.as_mut(),
            Self::Borrowed(value) => {
                // SAFETY: caller must ensure that value is a valid reference
                let value: &'s [T] = unsafe { value.as_ref() }.unwrap();

                let boxed: Box<[T]> = value.iter().cloned().collect();
                mem::replace(self, Self::new_boxed(boxed));

                match self {
                    Self::Owned(value) => value.as_mut(),
                    Self::Borrowed(_)  => unreachable!(),
                }
            }
        }
    }
}
