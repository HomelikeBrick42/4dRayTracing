import include.camera;
import include.ray;
import include.sdf;
import include.wormhole;
import include.objects_info;

[vk::binding(0, 0)]
[format("bgra8")]
WTexture2D output_texture;

[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

[vk::binding(0, 2)]
ConstantBuffer<ObjectsInfo> objects_info;

[vk::binding(1, 2)]
StructuredBuffer<Wormhole> wormholes;

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
    var total_distance = 0.0;
    for (int i = 0; i < 1000; i++)
    {
        let distance = abs(scene_sdf(ray.origin));

        if (distance < 0.001)
        {
            let scale = 50.0;
            let kind = sin(ray.origin.x * scale) + sin(ray.origin.y * scale) + sin(ray.origin.z * scale) + sin(ray.origin.w * scale);
            let normal = sdf::normal(scene_sdf, ray.origin).xyz * 0.5 + 0.5;
            color = normal * (kind > 0.0 ? float3(1.0) : float3(0.5));
            break;
        }

        ray.origin += ray.direction * distance;
        total_distance += distance;

        if (total_distance >= 1000.0)
            break;
    }

    output_texture.Store(global_index.xy, float4(color, 1.0));
}

float scene_sdf(float4 p)
{
    let throat_length = 4.0;
    let plane = abs(p.w) - throat_length;

    var d = plane;
    for (var i = 0; i < objects_info.wormholes_count; i++)
    {
        let wormhole = wormholes[i];
        let cylinder = length(p.xyz - wormhole.position) - (wormhole.throat_size + throat_length);
        d = max(d, -cylinder);
    }
    for (var i = 0; i < objects_info.wormholes_count; i++)
    {
        let wormhole = wormholes[i];
        let torus = sdf::torus(p - float4(wormhole.position, 0.0), wormhole.throat_size + throat_length, throat_length);
        d = min(d, torus);
    }

    return d;
}
