//! Mechanisms of hash-based key derive function (HKDF)
//! See: <https://docs.oasis-open.org/pkcs11/pkcs11-curr/v3.0/os/pkcs11-curr-v3.0-os.html#_Toc30061597>

use std::{convert::TryInto, marker::PhantomData, ptr::null_mut, slice};

use cryptoki_sys::{CKF_HKDF_SALT_DATA, CKF_HKDF_SALT_KEY, CKF_HKDF_SALT_NULL};

use crate::object::ObjectHandle;

use super::MechanismType;

#[derive(Debug, Clone, Copy)]
/// The salt for the extract stage.
pub enum HkdfSalt<'a> {
    /// CKF_HKDF_SALT_NULL no salt is supplied.
    Null,
    /// CKF_HKDF_SALT_DATA salt is supplied as a data in pSalt with length ulSaltLen.
    Data(&'a [u8]),
    /// CKF_HKDF_SALT_KEY salt is supplied as a key in hSaltKey
    Key(ObjectHandle),
}

/// HKDF parameters.
///
/// This structure wraps a `CK_HKDF_PARAMS` structure.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct HkdfParams<'a> {
    inner: cryptoki_sys::CK_HKDF_PARAMS,
    /// Marker type to ensure we don't outlive the data
    _marker: PhantomData<&'a [u8]>,
}

impl<'a> HkdfParams<'a> {
    /// Construct parameters for hash-based key derive function (HKDF).
    ///
    /// # Arguments
    ///
    /// * `extract` - Whether to execute the extract portion of HKDF.
    ///
    /// * `expand` - Whether to execute the expand portion of HKDF.
    ///
    /// * `prf_hash_mechanism` - The base hash used for the HMAC in the underlying HKDF operation
    ///
    /// * `salt` - The salt for the extract stage.
    ///
    /// * `info` - The info string for the expand stage.
    pub fn new(
        extract: bool,
        expand: bool,
        prf_hash_mechanism: MechanismType,
        salt: HkdfSalt,
        info: &'a [u8],
    ) -> Self {
        Self {
            inner: cryptoki_sys::CK_HKDF_PARAMS {
                bExtract: extract as u8,
                bExpand: expand as u8,
                prfHashMechanism: *prf_hash_mechanism,
                ulSaltType: match salt {
                    HkdfSalt::Null => CKF_HKDF_SALT_NULL,
                    HkdfSalt::Data(_) => CKF_HKDF_SALT_DATA,
                    HkdfSalt::Key(_) => CKF_HKDF_SALT_KEY,
                },
                pSalt: match salt {
                    HkdfSalt::Data(data) => data.as_ptr() as *mut _,
                    _ => null_mut(),
                },
                ulSaltLen: match salt {
                    HkdfSalt::Data(data) => data
                        .len()
                        .try_into()
                        .expect("salt length does not fit in CK_ULONG"),
                    _ => 0,
                },
                hSaltKey: match salt {
                    HkdfSalt::Key(key) => key.handle(),
                    _ => 0,
                },
                pInfo: info.as_ptr() as *mut _,
                ulInfoLen: info
                    .len()
                    .try_into()
                    .expect("info length does not fit in CK_ULONG"),
            },
            _marker: PhantomData,
        }
    }

    /// Whether to execute the extract portion of HKDF.
    pub fn extract(&self) -> bool {
        self.inner.bExtract != 0
    }

    /// Whether to execute the expand portion of HKDF.
    pub fn expand(&self) -> bool {
        self.inner.bExpand != 0
    }

    /// The salt for the extract stage.
    pub fn salt(&self) -> HkdfSalt<'a> {
        match self.inner.ulSaltType {
            CKF_HKDF_SALT_NULL => HkdfSalt::Null,
            CKF_HKDF_SALT_DATA => HkdfSalt::Data(unsafe {
                slice::from_raw_parts(self.inner.pSalt, self.inner.ulSaltLen as _)
            }),
            CKF_HKDF_SALT_KEY => HkdfSalt::Key(ObjectHandle::new(self.inner.hSaltKey)),
            _ => unreachable!(),
        }
    }

    /// The info string for the expand stage.
    pub fn info(&self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self.inner.pInfo, self.inner.ulInfoLen as _) }
    }
}
