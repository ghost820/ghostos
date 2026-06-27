use core::marker::PhantomData;

use x86_64::instructions::port::{Port, PortRead, PortWrite};

#[derive(Clone, Copy)]
pub struct ReadOnly;
#[derive(Clone, Copy)]
pub struct WriteOnly;
#[derive(Clone, Copy)]
pub struct ReadWrite;

pub trait CanRead {}
pub trait CanWrite {}

impl CanRead for ReadOnly {}
impl CanRead for ReadWrite {}

impl CanWrite for WriteOnly {}
impl CanWrite for ReadWrite {}

#[derive(Clone, Copy)]
pub struct PortAddress<T, Access> {
    number: u16,
    marker: PhantomData<(T, Access)>,
}

impl<T, Access> PortAddress<T, Access> {
    pub(crate) const unsafe fn new(number: u16) -> Self {
        Self {
            number,
            marker: PhantomData,
        }
    }
}

pub fn read<T, Access>(address: PortAddress<T, Access>) -> T
where
    T: PortRead,
    Access: CanRead,
{
    let mut port = Port::new(address.number);

    unsafe { port.read() }
}

pub fn write<T, Access>(address: PortAddress<T, Access>, value: T)
where
    T: PortWrite,
    Access: CanWrite,
{
    let mut port = Port::new(address.number);

    unsafe {
        port.write(value);
    }
}
