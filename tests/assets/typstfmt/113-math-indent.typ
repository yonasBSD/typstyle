$
    EE_(i_k sim "Unif"({1,...,d}))[f(xx_(k+1)) - f(xx_k)] 
        &<= sum_(i=1)^d 1/d (delta nabla_i f(x_k) + L_i/2 delta^2) \
        &= 1/d sum_(i=1)^d (-alpha_i_k nabla_i f(x_k)^2 + alpha^2_i_k L_i/2 nabla_i f(x_k)^2) \
        &= 1/d sum_(i=1)^d (-alpha_i_k + alpha^2_i_k L_i/2) nabla_i f(x_k)^2 \
$