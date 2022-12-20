use crate::{
    cap_type, sys, CapRights, CapType, IPCBuffer, InvocationContext, ObjectBlueprint,
    ObjectBlueprintAArch64, ObjectBlueprintArm, Result, Unspecified, VMAttributes, PGD,
};

pub const GRANULE: FrameSize = FrameSize::Small;

impl FrameSize {
    pub fn blueprint(self) -> ObjectBlueprint {
        match self {
            FrameSize::Small => ObjectBlueprintArm::SmallPage.into(),
            FrameSize::Large => ObjectBlueprintArm::LargePage.into(),
            FrameSize::Huge => ObjectBlueprintAArch64::HugePage.into(),
        }
    }

    pub const fn bits(self) -> usize {
        match self {
            FrameSize::Small => 12,
            FrameSize::Large => 21,
            FrameSize::Huge => 30,
        }
    }

    pub const fn bytes(self) -> usize {
        1 << self.bits()
    }
}

pub trait FrameType: CapType {
    const FRAME_SIZE: FrameSize;
}

impl FrameType for cap_type::SmallPage {
    const FRAME_SIZE: FrameSize = FrameSize::Small;
}

impl FrameType for cap_type::LargePage {
    const FRAME_SIZE: FrameSize = FrameSize::Large;
}

impl FrameType for cap_type::HugePage {
    const FRAME_SIZE: FrameSize = FrameSize::Huge;
}

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    Small,
    Large,
    Huge,
}

pub const LEVEL_BITS: usize = 9;

pub trait IntermediateTranslationStructureType: CapType {
    const SPAN_BITS: usize;
    const SPAN_BYTES: usize = 1 << Self::SPAN_BITS;

    fn _map_raw(
        ipc_buffer: &mut IPCBuffer,
        service: sys::seL4_CPtr,
        vspace: sys::seL4_CPtr,
        vaddr: sys::seL4_Word,
        attr: sys::seL4_ARM_VMAttributes::Type,
    ) -> sys::seL4_Error::Type;
}

impl IntermediateTranslationStructureType for cap_type::PUD {
    const SPAN_BITS: usize = cap_type::PD::SPAN_BITS + LEVEL_BITS;

    fn _map_raw(
        ipc_buffer: &mut IPCBuffer,
        service: sys::seL4_CPtr,
        vspace: sys::seL4_CPtr,
        vaddr: sys::seL4_Word,
        attr: sys::seL4_ARM_VMAttributes::Type,
    ) -> sys::seL4_Error::Type {
        ipc_buffer
            .inner_mut()
            .seL4_ARM_PageUpperDirectory_Map(service, vspace, vaddr, attr)
    }
}

impl IntermediateTranslationStructureType for cap_type::PD {
    const SPAN_BITS: usize = cap_type::PT::SPAN_BITS + LEVEL_BITS;

    fn _map_raw(
        ipc_buffer: &mut IPCBuffer,
        service: sys::seL4_CPtr,
        vspace: sys::seL4_CPtr,
        vaddr: sys::seL4_Word,
        attr: sys::seL4_ARM_VMAttributes::Type,
    ) -> sys::seL4_Error::Type {
        ipc_buffer
            .inner_mut()
            .seL4_ARM_PageDirectory_Map(service, vspace, vaddr, attr)
    }
}

impl IntermediateTranslationStructureType for cap_type::PT {
    const SPAN_BITS: usize = FrameSize::Small.bits() + LEVEL_BITS;

    fn _map_raw(
        ipc_buffer: &mut IPCBuffer,
        service: sys::seL4_CPtr,
        vspace: sys::seL4_CPtr,
        vaddr: sys::seL4_Word,
        attr: sys::seL4_ARM_VMAttributes::Type,
    ) -> sys::seL4_Error::Type {
        ipc_buffer
            .inner_mut()
            .seL4_ARM_PageTable_Map(service, vspace, vaddr, attr)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AnyFrame<C> {
    cptr: Unspecified<C>,
    size: FrameSize,
}

impl<C> AnyFrame<C> {
    pub fn cptr(&self) -> &Unspecified<C> {
        &self.cptr
    }

    pub fn size(&self) -> &FrameSize {
        &self.size
    }
}

impl<C: InvocationContext> AnyFrame<C> {
    pub fn map(self, pgd: PGD, vaddr: usize, rights: CapRights, attrs: VMAttributes) -> Result<()> {
        match self.size() {
            FrameSize::Small => self
                .cptr
                .downcast::<cap_type::SmallPage>()
                .map(pgd, vaddr, rights, attrs),
            FrameSize::Large => self
                .cptr
                .downcast::<cap_type::LargePage>()
                .map(pgd, vaddr, rights, attrs),
            FrameSize::Huge => self
                .cptr
                .downcast::<cap_type::HugePage>()
                .map(pgd, vaddr, rights, attrs),
        }
    }

    pub fn unmap(self) -> Result<()> {
        match self.size() {
            FrameSize::Small => self.cptr.downcast::<cap_type::SmallPage>().unmap(),
            FrameSize::Large => self.cptr.downcast::<cap_type::LargePage>().unmap(),
            FrameSize::Huge => self.cptr.downcast::<cap_type::HugePage>().unmap(),
        }
    }

    pub fn get_address(self) -> Result<usize> {
        match self.size() {
            FrameSize::Small => self.cptr.downcast::<cap_type::SmallPage>().get_address(),
            FrameSize::Large => self.cptr.downcast::<cap_type::LargePage>().get_address(),
            FrameSize::Huge => self.cptr.downcast::<cap_type::HugePage>().get_address(),
        }
    }
}
