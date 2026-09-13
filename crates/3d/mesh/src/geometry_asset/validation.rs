//! Validate reader preconditions before calling gltf's debug-asserting typed iterators.
pub(super) fn layout(document: &gltf::Gltf, buffers: &[Vec<u8>]) -> Result<(), String> {
    fn bounds(
        view: gltf::buffer::View<'_>,
        offset: usize,
        count: usize,
        size: usize,
    ) -> Result<(), String> {
        let stride = view.stride().unwrap_or(size);
        let end = count
            .checked_sub(1)
            .and_then(|n| n.checked_mul(stride))
            .and_then(|n| n.checked_add(size))
            .and_then(|n| n.checked_add(offset));
        if stride < size || end.is_none_or(|end| end > view.length()) {
            return Err("accessor exceeds buffer view or has invalid stride/count".into());
        }
        Ok(())
    }
    for accessor in document.accessors() {
        if accessor
            .count()
            .checked_mul(accessor.size())
            .is_none_or(|n| n > 128 * 1024 * 1024)
        {
            return Err("decoded accessor exceeds import budget".into());
        }
        if let Some(view) = accessor.view() {
            bounds(view, accessor.offset(), accessor.count(), accessor.size())?;
        }
        if let Some(sparse) = accessor.sparse() {
            if sparse.count() > accessor.count() {
                return Err("sparse accessor count exceeds base count".into());
            }
            let indices = sparse.indices();
            let size = indices.index_type().size();
            let view = indices.view();
            bounds(view.clone(), indices.offset(), sparse.count(), size)?;
            bounds(
                sparse.values().view(),
                sparse.values().offset(),
                sparse.count(),
                accessor.size(),
            )?;
            let start = view.offset() + indices.offset();
            let stride = view.stride().unwrap_or(size);
            let data = &buffers[view.buffer().index()];
            let mut previous = None;
            for i in 0..sparse.count() {
                let bytes = &data[start + i * stride..start + i * stride + size];
                let index = match size {
                    1 => bytes[0] as usize,
                    2 => u16::from_le_bytes(bytes.try_into().unwrap()) as usize,
                    4 => u32::from_le_bytes(bytes.try_into().unwrap()) as usize,
                    _ => unreachable!(),
                };
                if index >= accessor.count() || previous.is_some_and(|p| index <= p) {
                    return Err("sparse indices must increase within accessor bounds".into());
                }
                previous = Some(index);
            }
        } else if accessor.view().is_none() {
            return Err("accessor has no data".into());
        }
    }
    Ok(())
}

pub(super) fn typed(
    accessor: gltf::Accessor<'_>,
    dimensions: gltf::accessor::Dimensions,
    types: &[gltf::accessor::DataType],
) -> Result<(), String> {
    if matches!(
        accessor.data_type(),
        gltf::accessor::DataType::F32 | gltf::accessor::DataType::U32
    ) && accessor.normalized()
    {
        return Err("float/unsigned-int accessor cannot be normalized".into());
    }
    if accessor.dimensions() != dimensions || !types.contains(&accessor.data_type()) {
        return Err(format!("invalid type for accessor {}", accessor.index()));
    }
    Ok(())
}

pub(super) fn normalized(
    accessor: gltf::Accessor<'_>,
    dimensions: gltf::accessor::Dimensions,
    types: &[gltf::accessor::DataType],
) -> Result<(), String> {
    typed(accessor.clone(), dimensions, types)?;
    if accessor.data_type() != gltf::accessor::DataType::F32 && !accessor.normalized() {
        return Err("integer weights/rotation accessor must be normalized".into());
    }
    Ok(())
}
