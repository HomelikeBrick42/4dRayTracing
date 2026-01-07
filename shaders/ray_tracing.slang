import include.camera;
import include.ray;
import include.sdf;
import include.objects_info;
import include.wormhole;
import include.spheres;

[vk::binding(0, 0)]
[format("bgra8")]
WTexture2D output_texture;

[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

[vk::binding(0, 2)]
ConstantBuffer<ObjectsInfo> objects_info;

[vk::binding(1, 2)]
StructuredBuffer<Wormhole> wormholes;

[vk::binding(2, 2)]
StructuredBuffer<Sphere> spheres;

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

    var hit = hit_scene(ray);

    for (var i = 0; i < objects_info.spheres_count; i++)
    {
        let new_hit = hit_sphere(spheres[i], ray);
        if (new_hit.hasValue && (!hit.hasValue || (hit.hasValue && new_hit.value.distance < hit.value.distance)))
            hit = new_hit;
    }

    if (hit.hasValue)
        color = hit.value.color;

    output_texture.Store(global_index.xy, float4(color, 1.0));
}

Optional<Hit> hit_sphere(Sphere sphere, Ray ray)
{
    let sphere_radius = 0.5;

    let oc = sphere.position - ray.origin;
    // TODO: can this be replaced with 1?
    let a = dot(ray.direction, ray.direction);
    let h = dot(ray.direction, oc);
    let c = dot(oc, oc) - sphere_radius * sphere_radius;
    let discriminant = h * h - a * c;

    if (discriminant < 0.0)
        return none;

    let d1 = (h - sqrt(discriminant)) / a;
    // let d2 = (h + sqrt(discriminant)) / a;

    if (d1 < 0.0)
        return none;

    var hit : Hit;
    hit.distance = d1;
    hit.position = ray.origin + ray.direction * hit.distance;

    let scale = 100.0;

    let relative_hit_pos = hit.position - sphere.position;
    let projected_hit_pos = float4(dot(relative_hit_pos, sphere.forward), dot(relative_hit_pos, sphere.up), dot(relative_hit_pos, sphere.right), dot(relative_hit_pos, sphere.ana));
    let kind = sin(projected_hit_pos.x * scale) + sin(projected_hit_pos.y * scale) + sin(projected_hit_pos.z * scale) + sin(projected_hit_pos.w * scale);

    let normal = relative_hit_pos / sphere_radius;
    let projected_normal = projected_hit_pos / sphere_radius;

    hit.color = (projected_normal.xyz * 0.5 + 0.5) * (kind > 0.0 ? float3(1.0) : float3(0.5));
    return hit;
}

Optional<Hit> hit_scene(Ray ray)
{
    var total_distance = 0.0;
    for (int i = 0; i < 1000; i++)
    {
        let distance = abs(scene_sdf(ray.origin));

        if (distance < 0.001)
        {
            let scale = 50.0;
            let kind = sin(ray.origin.x * scale) + sin(ray.origin.y * scale) + sin(ray.origin.z * scale) + sin(ray.origin.w * scale);
            let color = sdf::normal(scene_sdf, ray.origin).xyz * 0.5 + 0.5;

            var hit : Hit;
            hit.position = ray.origin;
            hit.color = color * (kind > 0.0 ? float3(1.0) : float3(0.5));
            hit.distance = total_distance;
            return hit;
        }

        ray.origin += ray.direction * distance;
        total_distance += distance;

        if (total_distance >= 1000.0)
            break;
    }
    return none;
}

// float scene_sdf(float4 p)
// {
//     let throat_length = 4.0;
//     let plane = abs(p.w) - throat_length;

//     var d = plane;
//     for (var i = 0; i < objects_info.wormholes_count; i++)
//     {
//         let wormhole = wormholes[i];
//         let cylinder = length(p.xyz - wormhole.position) - (wormhole.throat_size + throat_length);
//         d = max(d, -cylinder);
//     }
//     for (var i = 0; i < objects_info.wormholes_count; i++)
//     {
//         let wormhole = wormholes[i];
//         let torus = sdf::torus(p - float4(wormhole.position, 0.0), wormhole.throat_size + throat_length, throat_length);
//         d = min(d, torus);
//     }

//     return d;
// }

float scene_sdf(float4 p)
{
    return sdf::wormholes(p, wormholes, objects_info.wormholes_count, objects_info.plane_height, objects_info.join_position);
}
