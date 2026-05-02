extern crate alloc;

use crate::reproducibility::Reproducibility;
use crate::FastBlockRng;
use alloc::boxed::Box;
use rand_core::block::Generator;
use rand_core::utils::Word;
use serde::{Deserialize, Serialize};

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CoreState<T, W: Word> {
    core: T,
    remaining_results: Box<[W]>,
}

impl<R: Reproducibility, W: Word + Serialize, const N: usize, G: Serialize + Generator<Output = [W; N]> + Clone> serde::Serialize for FastBlockRng<R, W, N, G> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        CoreState {
            core: self.block_core.core.clone(),
            remaining_results: self
                .block_core
                .remaining_results()
                .to_vec()
                .into_boxed_slice(),
        }
        .serialize(serializer)
    }
}

impl<'de, R: Reproducibility, W: Word + Deserialize<'de>, const N: usize, G: Deserialize<'de> + Generator<Output = [W; N]> + Clone> serde::Deserialize<'de> for FastBlockRng<R, W, N, G> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::marker::PhantomData;
        use rand_core::block::BlockRng;
        let CoreState {core, remaining_results}: CoreState<G, W> = CoreState::deserialize(deserializer)?;
        if let Some(block_core) = BlockRng::reconstruct(core.clone(), &remaining_results) {
            Ok(FastBlockRng {
                block_core,
                reproducibility: PhantomData,
            })
        } else {
            Ok(FastBlockRng::from_core(core))
        }
    }
}
