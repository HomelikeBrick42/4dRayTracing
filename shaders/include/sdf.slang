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

float torus(float4 p, float ring_radius, float radius)
{
    let position = float4(0.0, 0.0, 0.0, 0.0);
    let q = float2(length((p - position).xyz) - ring_radius, (p - position).w);
    return length(q) - radius;
}

}
