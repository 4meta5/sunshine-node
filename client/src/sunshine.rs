use crate::error::Error;
#[cfg(feature = "light-client")]
use crate::light_client::ChainType;
use crate::runtime::{Client, Pair, Runtime, XtBuilder};
use crate::srml::shares_atomic::*;
use ipfs_embed::{Config, Store};
use ipld_block_builder::BlockBuilder;
use sp_core::Pair as _;
use std::path::Path;
use substrate_subxt::system::*;

pub struct Sunshine {
    account_id: <Runtime as System>::AccountId,
    subxt: Client,
    xt: XtBuilder,
    ipld: BlockBuilder<Store>,
}

impl Sunshine {
    #[cfg(feature = "light-client")]
    pub async fn new<T: AsRef<Path>>(
        path: T,
        signer: Pair,
        chain: ChainType,
    ) -> Result<Self, Error> {
        let db = sled::open(path)?;
        let ipld_tree = db.open_tree("ipld_tree")?;
        let subxt_tree = db.open_tree("subxt_tree")?;
        let config = Config::from_tree(ipld_tree);
        let store = Store::new(config)?;
        let ipld = BlockBuilder::new(store);
        let account_id = signer.public().into();
        let subxt = crate::light_client::build_light_client(subxt_tree, chain).await?;
        let xt = subxt.xt(signer, None).await?;
        Ok(Self {
            account_id,
            subxt,
            xt,
            ipld,
        })
    }

    #[cfg(not(feature = "light-client"))]
    pub async fn new<T: AsRef<Path>>(path: T, signer: Pair) -> Result<Self, Error> {
        let db = sled::open(path)?;
        let ipld_tree = db.open_tree("ipld_tree")?;
        let config = Config::from_tree(ipld_tree);
        let store = Store::new(config)?;
        let ipld = BlockBuilder::new(store);
        let account_id = signer.public().into();
        let subxt = crate::runtime::ClientBuilder::new().build().await?;
        let xt = subxt.xt(signer, None).await?;
        Ok(Self {
            account_id,
            subxt,
            xt,
            ipld,
        })
    }

    pub async fn reserve_shares(
        &self,
        org: u32,
        share: u32,
    ) -> Result<SharesReservedEvent<Runtime>, Error> {
        self.xt
            .clone()
            .watch()
            .reserve_shares(org, share, &self.account_id)
            .await?
            .shares_reserved()
            .map_err(|e| substrate_subxt::Error::Codec(e))?
            .ok_or(Error::EventNotFound)
    }
}
