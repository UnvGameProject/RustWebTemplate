CREATE TABLE public.users (
                              id UUID PRIMARY KEY,
                              email TEXT NOT NULL
                                  CHECK (char_length(email) BETWEEN 3 AND 320),
                              password_hash TEXT NOT NULL
                                  CHECK (char_length(password_hash) > 0),
                              created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                              updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX users_email_unique
    ON public.users (lower(email));

CREATE TABLE public.user_sessions (
                                      token_hash BYTEA PRIMARY KEY
                                          CHECK (octet_length(token_hash) = 32),
                                      user_id UUID NOT NULL
                                          REFERENCES public.users(id)
                                              ON DELETE CASCADE,
                                      expires_at TIMESTAMPTZ NOT NULL,
                                      created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                                      updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX user_sessions_user_id_index
    ON public.user_sessions (user_id);

CREATE INDEX user_sessions_expires_at_index
    ON public.user_sessions (expires_at);