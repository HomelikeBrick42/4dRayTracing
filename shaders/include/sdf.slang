namespace sdf
{

float sphere(float4 position)
{
    let sphere_position = float4(0.0, 0.0, 0.0, 0.0);
    let sphere_radius = 1.0;
    return length(position - sphere_position) - sphere_radius;
}

}
