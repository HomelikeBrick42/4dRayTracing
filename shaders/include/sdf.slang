namespace sdf
{

float4 normal(functype(float4)->float f, float4 p)
{
    let helper = float2(0.001, 0.0);
    return normalize(float4(
        f(p + helper.xyyy) - f(p - helper.xyyy),
        f(p + helper.yxyy) - f(p - helper.yxyy),
        f(p + helper.yyxy) - f(p - helper.yyxy),
        f(p + helper.yyyx) - f(p - helper.yyyx)));
}

float sphere(float4 p)
{
    let position = float4(0.0, 0.0, 0.0, 0.0);
    let radius = 1.0;
    return length(p - position) - radius;
}

float torus(float4 p, float ring_radius, float radius)
{
    let position = float4(0.0, 0.0, 0.0, 0.0);
    let q = float2(length((p - position).xzw) - ring_radius, (p - position).y);
    return length(q) - radius;
}

float cylinder(float4 p)
{
    let radius = 0.6;
    let height = 1.0;
    let d = abs(float2(length(p.xzw), p.y)) - float2(radius, height);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

}
