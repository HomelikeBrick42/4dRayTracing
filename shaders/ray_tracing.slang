import include.camera;
import include.ray;
import include.sdf;

[vk::binding(0, 0)]
[format("bgra8")]
WTexture2D output_texture;

[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

[shader("compute")]
[numthreads(16, 16, 1)]
void trace_rays(uint3 global_index: SV_DispatchThreadID)
{
    var width : uint;
    var height : uint;
    output_texture.GetDimensions(width, height);

    if (global_index.x >= width || global_index.y >= height)
        return;

    let aspect = float(width) / float(height);
    let uv = ((float2(global_index.xy) + 0.5) / float2(width, height)) * 2.0 - 1.0;
    let fov_scale = tan(camera.fov * 0.5);

    var ray : Ray;
    ray.origin = camera.position;
    ray.direction = normalize(camera.forward + camera.up * uv.y * fov_scale + camera.right * uv.x * fov_scale * aspect);

    var color = float3(0.0);
    for (int i = 0; i < 100; i++)
    {
        let distance = sdf::sphere(ray.origin);

        if (distance < 0.001)
        {
            let scale = 50.0;
            let kind = sin(ray.origin.x * scale) + sin(ray.origin.y * scale) + sin(ray.origin.z * scale) + sin(ray.origin.w * scale);
            color = kind > 0.0 ? float3(1.0) : float3(0.5);
            break;
        }

        ray.origin += ray.direction * distance;
    }

    output_texture.Store(global_index.xy, float4(color, 1.0));
}
