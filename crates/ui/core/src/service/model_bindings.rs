impl UiModelBindingService {
    pub fn queue(&self, binding: UiModelBinding) {
        let mut bindings = self
            .bindings
            .lock()
            .expect("ui model binding service mutex should not be poisoned");
        if let Some(existing) = bindings
            .iter_mut()
            .find(|existing| existing.path == binding.path && existing.kind == binding.kind)
        {
            *existing = binding;
        } else {
            bindings.push(binding);
        }
    }

    pub fn bindings(&self) -> Vec<UiModelBinding> {
        self.bindings
            .lock()
            .expect("ui model binding service mutex should not be poisoned")
            .clone()
    }

    pub fn clear(&self) {
        self.bindings
            .lock()
            .expect("ui model binding service mutex should not be poisoned")
            .clear();
    }
}
