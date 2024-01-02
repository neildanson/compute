USeful compute inputs

```
      @builtin(workgroup_id) workgroup_id : vec3<u32>,
      @builtin(local_invocation_id) local_invocation_id : vec3<u32>,
      @builtin(global_invocation_id) global_invocation_id : vec3<u32>,
```

workgroup_id => 0..workgroup_size.x, 0..workgroup_size.y, 0..workgroup_size.z (decorated in shader)
local_invocation_id = 0..dispatch_size.x,  0..dispatch_size.y,  0..dispatch_size.z (set in code) 
global_invocation_id = workgroup_id * workgroup_size + local_invocation_id