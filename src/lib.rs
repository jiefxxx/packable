#[macro_export]
macro_rules! pack {
   ( $le:expr, $( $x:expr ),* ) => {
        {
            let litle_endian: bool = $le;
            let mut temp_vec = Vec::new();
            $(
                temp_vec.extend_from_slice(Packable::pack(&$x, litle_endian).as_slice());
            )*
            temp_vec
        }
    };
}

#[macro_export]
macro_rules! unpack {
    ( $le:expr, $buf:expr, $( $x:expr ),* ) => {
        {
            (|| {
                let litle_endian: bool = $le;
                let buffer: &mut Vec<u8> = $buf;
                $(
                    let size = Packable::size(&$x);
                    if buffer.len() >= size{
                        let split_buf = buffer.split_off(size);
                        Packable::unpack(&mut $x, buffer, litle_endian)?;
                        *buffer = split_buf;
                    }
                    else{
                        return Err(PackableError { 
                            error_kind: ErrorKind::BufferLengthError, 
                            data: format!("except {} bytes and get {}", size, buffer.len())
                        })
                    }
                )*
                Ok(())
            })()
        }
    };
}

use core::fmt;
use std::{mem, array::TryFromSliceError};

pub trait Packable {
    fn pack(&self, litle_endian: bool) -> Vec<u8>;
    fn size(&self) -> usize;
    fn unpack(&mut self, data: &mut Vec<u8>, litle_endian: bool) -> Result<(), PackableError>;
}

macro_rules! impl_packable_numerique {
    ( $le:ty ) => {
        impl Packable for $le {
            fn pack(&self, litle_endian: bool) -> Vec<u8>{
                if litle_endian{
                    self.to_le_bytes().to_vec()
                }
                else{
                    self.to_be_bytes().to_vec()
                }
             }
        
            fn size(&self) -> usize {
                mem::size_of::<$le>()
            }
        
            fn unpack(&mut self, data: &mut Vec<u8>, litle_endian: bool) -> Result<(), PackableError>{
                if litle_endian{
                    *self = <$le>::from_le_bytes(data[0..self.size()].try_into()?);
                }
                else{
                    *self = <$le>::from_be_bytes(data[0..self.size()].try_into()?);
                }
                Ok(())
            }
        }
     };
 }

impl_packable_numerique!(u8);
impl_packable_numerique!(u16);
impl_packable_numerique!(u32);
impl_packable_numerique!(u64);
impl_packable_numerique!(u128);
impl_packable_numerique!(i8);
impl_packable_numerique!(i16);
impl_packable_numerique!(i32);
impl_packable_numerique!(i64);
impl_packable_numerique!(i128);
impl_packable_numerique!(f32);
impl_packable_numerique!(f64);

impl<const DIMENSIONS: usize> Packable for [u8; DIMENSIONS]{
    fn pack(&self, _litle_endian: bool) -> Vec<u8> {
        self.to_vec()
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn unpack(&mut self, data: &mut Vec<u8>, _litle_endian: bool) -> Result<(), PackableError> {
        let value = &mut data[..self.size()];
        self.clone_from_slice(value);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Flag{
    base: u8,
}

impl Flag{
    pub fn set(&mut self, id: u8, value: bool){
        if value{
            self.base = self.base|(0x1<<id)
        }
        else{
            self.base = self.base&!(0x1<<id)
        }
    }

    pub fn get(&self, id: u8) -> bool{
        (self.base&(0x1<<id))>0
    }
}

impl Packable for Flag{
    fn pack(&self, litle_endian: bool) -> Vec<u8> {
        self.base.pack(litle_endian)
    }

    fn size(&self) -> usize {
        self.base.size()
    }

    fn unpack(&mut self, data: &mut Vec<u8>, litle_endian: bool) -> Result<(), PackableError> {
        self.base.unpack(data, litle_endian)
    }
}

#[derive(Debug)]
pub enum ErrorKind{
    TryFromSliceError,
    BufferLengthError,
}

#[derive(Debug)]
pub struct PackableError{
    pub error_kind: ErrorKind,
    pub data: String,
}

impl fmt::Display for PackableError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "packable error {:?} {}", self.error_kind, self.data)
    }
}

impl From<TryFromSliceError> for PackableError{
    fn from(error: TryFromSliceError) -> Self {
        PackableError { 
            error_kind: ErrorKind::TryFromSliceError, 
            data: format!("{}", error) 
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Packable;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn tast_pack_u8(){
        assert_eq!(vec![42u8], pack!(false, 42u8));
    }

    #[test]
    fn tast_pack_u16(){
        assert_eq!(vec![0, 42], pack!(false, 42u16));
        assert_eq!(vec![42, 0], pack!(true, 42u16));
    }

    #[test]
    fn tast_pack_u32(){
        assert_eq!(vec![0, 0, 0, 42], pack!(false, 42u32));
        assert_eq!(vec![42, 0, 0, 0], pack!(true, 42u32));
    }
    #[test]
    fn tast_pack_u64(){
        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 42], pack!(false, 42u64));
        assert_eq!(vec![42, 0, 0, 0, 0, 0, 0, 0], pack!(true, 42u64));
    }
    #[test]
    fn tast_pack_i8(){
        assert_eq!(vec![214], pack!(false, -42i8));
    }

    #[test]
    fn tast_pack_i16(){
        assert_eq!(vec![255, 214], pack!(false, -42i16));
        assert_eq!(vec![214, 255], pack!(true, -42i16));
    }

    #[test]
    fn tast_pack_i32(){
        assert_eq!(vec![255, 255, 255, 214], pack!(false, -42i32));
        assert_eq!(vec![214, 255, 255, 255], pack!(true, -42i32));
    }
    #[test]
    fn tast_pack_i64(){
        assert_eq!(vec![255, 255, 255, 255, 255, 255, 255, 214], pack!(false, -42i64));
        assert_eq!(vec![214, 255, 255, 255, 255, 255, 255, 255], pack!(true, -42i64));
    }

    #[test]
    fn tast_pack_f32(){
        assert_eq!(vec![66, 42, 204, 205], pack!(false, 42.7f32));
        assert_eq!(vec![205, 204, 42, 66], pack!(true, 42.7f32));
    }
    #[test]
    fn tast_pack_f64(){
        assert_eq!(vec![64, 69, 94, 184, 81, 235, 133, 31], pack!(false, 42.74f64));
        assert_eq!(vec![31, 133, 235, 81, 184, 94, 69, 64], pack!(true, 42.74f64));
    }
}
