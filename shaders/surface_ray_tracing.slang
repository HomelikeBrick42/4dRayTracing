import include.surface_camera;
import include.rotor;
import include.ray;
import include.sdf;
import include.objects_info;
import include.render_settings;
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

[vk::binding(3, 2)]
ConstantBuffer<RenderSettings> render_settings;

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

    let forward = float3(1.0, 0.0, 0.0);
    let up = float3(0.0, 1.0, 0.0);
    let right = float3(0.0, 0.0, 1.0);
    let direction = normalize(forward + up * uv.y * fov_scale + right * uv.x * fov_scale * aspect);

    var color = float3(0.0);

    let hit = hit_scene(camera.position, camera.rotation, direction);
    if (hit.hasValue)
        color = hit.value.color;

    output_texture.Store(global_index.xy, float4(color, 1.0));
}

Optional<Hit> hit_scene(float4 position, Rotor rotation, float3 direction)
{
    var total_distance = 0.0;
    for (int i = 0; i < 100; i++)
    {
        var step_distance = float.maxValue;
        for (int sphere_index = 0; sphere_index < objects_info.spheres_count; sphere_index++)
        {
            let sphere = spheres[sphere_index];
            let sphere_radius = 0.5;

            let distance = length(sphere.position - position) - sphere_radius;
            if (distance < 0.001)
            {
                var hit : Hit;
                hit.position = position;
                hit.distance = total_distance;

                let scale = 100.0;

                let relative_hit_pos = hit.position - sphere.position;
                let projected_hit_pos = float4(dot(relative_hit_pos, sphere.forward), dot(relative_hit_pos, sphere.up), dot(relative_hit_pos, sphere.right), dot(relative_hit_pos, sphere.ana));
                let kind = sin(projected_hit_pos.x * scale) + sin(projected_hit_pos.y * scale) + sin(projected_hit_pos.z * scale) + sin(projected_hit_pos.w * scale);

                let normal = relative_hit_pos / sphere_radius;
                let projected_normal = projected_hit_pos / sphere_radius;

                hit.color = (projected_normal.xyz * 0.5 + 0.5) * (kind > 0.0 ? float3(1.0) : float3(0.5));

                return hit;
            }

            step_distance = min(step_distance, distance);
        }

        total_distance += step_distance;
        if (total_distance > 30.0)
            break;

        walk(position, rotation, direction, step_distance);
    }
    return none;
}

void walk(inout float4 position, inout Rotor rotation, float3 direction, float distance)
{
    while (distance > 0.0)
    {
        let step = min(distance, 0.1);
        distance -= step;

        position += rotation.rotate_direction(float4(direction * step, 0.0));
        project(position, rotation);
    }
}

void project(inout float4 position, inout Rotor rotation)
{
    let distance = scene_sdf(position);
    if (abs(distance) < 0.0001)
        return;

    var normal = sdf::normal(scene_sdf, position);
    position -= normal * distance;

    if (dot(normal, normal) > 0.0)
    {
        let old_normal = rotation.w();
        if (dot(normal, old_normal) < 0.0)
            normal *= -1.0;
        let correction_rotation = Rotor.from_to_vector(old_normal, normal);
        rotation = correction_rotation.then(rotation);
    }
}

float scene_sdf(float4 p)
{
    float d = float.maxValue;
    Optional<float> wormholes_signed_dist = sdf::wormholes(p, wormholes, objects_info.wormholes_count, objects_info.plane_height, objects_info.join_position);

    if (wormholes_signed_dist.hasValue)
    {
        d = wormholes_signed_dist.value;
    }
    else
    {
        d = sdf::cut_plane(float2(p.x - objects_info.join_position - objects_info.plane_height, p.w), objects_info.plane_height, objects_info.plane_height);
    }
    return d - render_settings.hit_offset;
}
