-- Canonical policy records share EdgeQuake's PostgreSQL/auth/workspace boundary.
CREATE TABLE public.tacitus_records (
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('source','assertion','method','review','event')),
    id UUID NOT NULL,
    version INTEGER NOT NULL CHECK (version > 0),
    payload JSONB NOT NULL CHECK (jsonb_typeof(payload) = 'object'),
    seq BIGSERIAL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, workspace_id, kind, id, version)
);
CREATE INDEX tacitus_records_scope_idx ON public.tacitus_records(tenant_id, workspace_id, kind, seq DESC);
CREATE INDEX tacitus_sources_document_idx ON public.tacitus_records((payload->>'document_id')) WHERE kind='source';
CREATE TABLE public.tacitus_analyses (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    user_id UUID NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('queued','retrieving','generating','completed','failed','cancelled','interrupted')),
    request JSONB NOT NULL,
    context JSONB,
    result JSONB,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX tacitus_analyses_scope_idx ON public.tacitus_analyses(tenant_id,workspace_id,created_at DESC);
-- Server code always binds both dimensions; RLS additionally protects direct scoped access.
ALTER TABLE public.tacitus_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.tacitus_analyses ENABLE ROW LEVEL SECURITY;
CREATE POLICY tacitus_records_scope ON public.tacitus_records USING (
 tenant_id::text=current_setting('app.tenant_id',true) AND workspace_id::text=current_setting('app.workspace_id',true)
) WITH CHECK (tenant_id::text=current_setting('app.tenant_id',true) AND workspace_id::text=current_setting('app.workspace_id',true));
CREATE POLICY tacitus_analyses_scope ON public.tacitus_analyses USING (
 tenant_id::text=current_setting('app.tenant_id',true) AND workspace_id::text=current_setting('app.workspace_id',true)
) WITH CHECK (tenant_id::text=current_setting('app.tenant_id',true) AND workspace_id::text=current_setting('app.workspace_id',true));

CREATE FUNCTION public.tacitus_withdraw_deleted_document() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE scoped_source RECORD;
BEGIN
 FOR scoped_source IN SELECT DISTINCT tenant_id,workspace_id,id FROM public.tacitus_records WHERE kind='source' AND payload->>'document_id'=OLD.id::text ORDER BY tenant_id,workspace_id,id LOOP
  PERFORM pg_advisory_xact_lock(hashtextextended(scoped_source.tenant_id::text||':'||scoped_source.workspace_id::text||':'||scoped_source.id::text,0));
 END LOOP;
 UPDATE public.tacitus_records SET payload=jsonb_set(payload,'{withdrawn}','true'::jsonb)
 WHERE kind='source' AND payload->>'document_id'=OLD.id::text;
 RETURN OLD;
END;
$$;
CREATE TRIGGER tacitus_document_withdraw BEFORE DELETE ON public.documents
FOR EACH ROW EXECUTE FUNCTION public.tacitus_withdraw_deleted_document();
