use pallas::{
    codec::minicbor::decode,
    ledger::primitives::{alonzo, byron},
    network::miniprotocols::{chainsync::HeaderContent, Point},
};

use crate::{model::MultiEraBlock, Error};

#[derive(Debug)]
pub enum MultiEraHeader {
    ByronBoundary(byron::EbbHead),
    Byron(byron::BlockHead),
    AlonzoCompatible(alonzo::Header),
}

impl TryFrom<HeaderContent> for MultiEraHeader {
    type Error = decode::Error;

    fn try_from(value: HeaderContent) -> Result<Self, Self::Error> {
        match value.variant {
            0 => match value.byron_prefix {
                Some((0, _)) => {
                    let header = decode(&value.cbor)?;
                    Ok(MultiEraHeader::ByronBoundary(header))
                }
                _ => {
                    let header = decode(&value.cbor)?;
                    Ok(MultiEraHeader::Byron(header))
                }
            },
            _ => {
                let header = decode(&value.cbor)?;
                Ok(MultiEraHeader::AlonzoCompatible(header))
            }
        }
    }
}

impl MultiEraHeader {
    pub fn read_cursor(&self) -> Result<Point, Error> {
        match self {
            MultiEraHeader::ByronBoundary(x) => {
                let hash = x.to_hash();
                let slot = x.to_abs_slot();
                Ok(Point::Specific(slot, hash.to_vec()))
            }
            MultiEraHeader::Byron(x) => {
                let hash = x.to_hash();
                let slot = x.consensus_data.0.to_abs_slot();
                Ok(Point::Specific(slot, hash.to_vec()))
            }
            MultiEraHeader::AlonzoCompatible(x) => {
                let hash = alonzo::crypto::hash_block_header(x);
                Ok(Point::Specific(x.header_body.slot, hash.to_vec()))
            }
        }
    }
}

#[derive(Debug)]
pub enum ChainSyncCommand {
    RollForward(Point),
    RollBack(Point),
}

impl ChainSyncCommand {
    pub fn roll_forward(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollForward(point),
        }
    }

    pub fn roll_back(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollBack(point),
        }
    }
}

#[derive(Debug)]
pub enum ChainSyncCommandEx {
    RollForward(MultiEraBlock),
    RollBack(Point),
}

impl ChainSyncCommandEx {
    pub fn roll_forward(block: MultiEraBlock) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollForward(block),
        }
    }

    pub fn roll_back(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollBack(point),
        }
    }
}
